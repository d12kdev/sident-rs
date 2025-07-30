#[cfg(not(target_os = "android"))]
compile_error!("SIACom is only compilable for Android!");


use std::{io, os::fd::{FromRawFd, OwnedFd}, pin::Pin, sync::Arc, task::Poll};

use futures::future::BoxFuture;
use jni::{objects::{JObject, JString, JValue}, sys::jint, JavaVM};
use log::info;
use nusb::{transfer::{ControlOut, ControlType, Direction, EndpointType, Recipient, RequestBuffer}, Device, Interface};
use tokio::{io::{AsyncRead, AsyncWrite}, sync::Mutex};

const SI_VID: u16 = 0x10C4;
const SI_PID: u16 = 0x800A;


fn jni_signature(args: Option<Vec<&str>>, ret: &str) -> String {
    fn map_type(typ: &str) -> String {
        match typ {
            "void" => "V".to_string(),
            "boolean" => "Z".to_string(),
            "byte" => "B".to_string(),
            "char" => "C".to_string(),
            "short" => "S".to_string(),
            "int" => "I".to_string(),
            "long" => "J".to_string(),
            "float" => "F".to_string(),
            "double" => "D".to_string(),
            _ if typ.ends_with("[]") => {
                let inner = &typ[..typ.len()-2];
                format!("[{}", map_type(inner))
            }
            _ => format!("L{};", typ.replace('.', "/")),
        }
    }

    let args_sig = args.unwrap_or_default()
        .iter()
        .map(|t| map_type(t))
        .collect::<String>();

    let ret_sig = map_type(ret);
    format!("({}){}", args_sig, ret_sig)
}

const REQCODE_SET_BAUDRATE: u8 = 0x1E;
const REQCODE_SET_LINE_CTL: u8 = 0x03;
const REQCODE_IFC_ENABLE: u8   = 0x00;

const DEFAULT_BAUD: u32 = 38400;

struct InnerSIAndroidCom {
    interface: Interface,
    in_ep: u8,
    out_ep: u8,
}

impl InnerSIAndroidCom {
    pub async fn new() -> io::Result<Self> {
        info!("Creating SI Android Communicaiton");
        let fd = tokio::task::spawn_blocking(move || -> io::Result<i32> {
            info!("Getting NDK Context");
            let _ctx = ndk_context::android_context();
            let _jvm_ptr = _ctx.vm();
            let _context = _ctx.context();
            info!("Creating CTX and JVM");
            let context = unsafe { JObject::from_raw(_context as *mut _) };
            let jvm = unsafe { JavaVM::from_raw(_jvm_ptr as *mut _) }.map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            info!("Attaching to JVM");
            let mut env = jvm.attach_current_thread().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            info!("Getting package name");
            let _package_name = env.call_method(
                &context,
                "getPackageName",
                jni_signature(None, "java.lang.String"),
                &[]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
            .l()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            let package_name: String = env.get_string(&JString::from(_package_name)).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.into();
            

            info!("Getting Context.USB_SERVICE");
            let _usb_service_str = env.get_static_field("android/content/Context", "USB_SERVICE", "Ljava/lang/String;")
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
                .l()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            let usb_service_str = JString::from(_usb_service_str);

            info!("Getting UsbManager");
            let usb_manager = env.call_method(
                &context,
                "getSystemService",
                jni_signature(Some(vec!["java.lang.String"]), "java.lang.Object"),
                &[JValue::Object(&usb_service_str)]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.l().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            info!("Getting device list");
            let device_list = env.call_method(
                &usb_manager,
                "getDeviceList",
                jni_signature(None, "java.util.HashMap"),
                &[]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.l().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            info!("Getting devices");
            let devices = env.call_method(
                device_list,
                "values",
                jni_signature(None, "java.util.Collection"),
                &[]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.l().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            info!("Creating iterator of devices");
            let iterator = env.call_method(
                devices,
                "iterator",
                jni_signature(None, "java.util.Iterator"),
                &[]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.l().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            let mut _dev: Option<JObject> = None;
            info!("Looping through devices...");
            while env.call_method(&iterator, "hasNext", jni_signature(None, "boolean"), &[])
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
                .z()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
            {
                let device = env.call_method(&iterator, "next", jni_signature(None, "java.lang.Object"), &[])
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
                    .l()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

                let device_vid = env.call_method(&device, "getVendorId", jni_signature(None, "int"), &[])
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
                    .i()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

                let device_pid = env.call_method(&device, "getProductId", jni_signature(None, "int"), &[])
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
                    .i()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

                if device_pid == SI_PID as i32 && device_vid == SI_VID as i32 {
                    _dev = Some(device);
                    break;
                }
            }

            let jdevice = _dev.ok_or(io::Error::new(io::ErrorKind::Other, format!("SI Device not found")))?;

            info!("Device found");

            info!("Requesting permissions");
            info!("Getting USB_PERMISSION string");
            let p_action_str = env.new_string(format!("{}.USB_PERMISSION", package_name)).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            info!("Creating Intent object");
            let p_intent_obj = env.new_object("android/content/Intent", jni_signature(Some(vec!["java.lang.String"]), "void"), &[JValue::from(&p_action_str)]).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            info!("getting intent - getBroadcast");
            let p_pending_intent = env.call_static_method(
                "android/app/PendingIntent",
                "getBroadcast",
                jni_signature(Some(vec!["android.content.Context", "int", "android.content.Intent", "int"]), "android.app.PendingIntent"),
                &[
                    JValue::from(&context),
                    jint::from(0).into(),
                    JValue::from(&p_intent_obj),
                    jint::from(0x04000000).into() // FLAG_IMMUTABLE
                ]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?
            .l()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            
            info!("Requesting USB permission");
            let _ = env.call_method(
                &usb_manager,
                "requestPermission",
                jni_signature(Some(vec!["android.hardware.usb.UsbDevice", "android.app.PendingIntent"]), "void"),
                &[JValue::from(&jdevice), JValue::from(&p_pending_intent)]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            info!("Permission requested. TODO: Check the outcome");

            info!("Opening device");

            let device_conn = env.call_method(
                &usb_manager,
                "openDevice",
                jni_signature(Some(vec!["android.hardware.usb.UsbDevice"]), "android.hardware.usb.UsbDeviceConnection"),
                &[JValue::Object(&jdevice)]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.l().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            if device_conn.is_null() {
                log::error!("Failed to open device connection");
                return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to open device connection")));
            }

            info!("Getting file descriptor");
            let fd = env.call_method(
                device_conn,
                "getFileDescriptor",
                jni_signature(None, "int"),
                &[]
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?.i().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            return Ok(fd);
        }).await??;

        info!("Creating owned fd");
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };

        info!("Creating Device");
        let device = Device::from_fd(owned_fd)?;
        info!("Getting config");
        let config = device.active_configuration()?;
        
        let mut iface_num: Option<u8> = None;
        let mut in_ep: Option<u8> = None;
        let mut out_ep: Option<u8> = None;
        info!("Getting interface");
        for iface_g in config.interfaces() {
            for alt_setings in iface_g.alt_settings() {
                for ep in alt_setings.endpoints() {
                    if ep.transfer_type() == EndpointType::Bulk {
                        match ep.direction() {
                            Direction::In => in_ep = Some(ep.address()),
                            Direction::Out => out_ep = Some(ep.address())
                        }
                    }
                }
                if in_ep.is_some() && out_ep.is_some() {
                    iface_num = Some(alt_setings.interface_number());
                    break;
                }
            }
            if iface_num.is_some() {
                break;
            }
        }

        let in_ep = in_ep.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No bulk IN endpoint found"))?;
        let out_ep = out_ep.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No bulk OUT endpoint found"))?;
        let iface_num = iface_num.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No suitable interface found"))?;

        info!("Claiming interface");
        let iface = device.claim_interface(iface_num)?;

        let this = Self {
            interface: iface,
            in_ep,
            out_ep,
        };

        this.setup_comm().await?;

        info!("Communication opened successfully");
        return Ok(this);
    }

    async fn setup_comm(&self) -> io::Result<()> {
        info!("Setting up communication");

        info!("Enabling UART");
        self.interface.control_out(ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Interface,
            request: REQCODE_IFC_ENABLE,
            value: 0x0001,                       // UART_ENABLE
            index: self.interface.interface_number() as u16,
            data: &[],
        }).await.into_result()?;

        self.set_baud_rate(DEFAULT_BAUD).await?;

        let mut config: u16 = 0;
        // 8 data bits
        config |= 0x0800;

        info!("Sending config");
        self.interface.control_out(ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Interface,
            request: REQCODE_SET_LINE_CTL,
            value: config,
            index: self.interface.interface_number() as u16,
            data: &[]
        }).await.into_result()?;

        return Ok(());
    }

    async fn bulkwrt(&mut self, buf: &[u8]) -> io::Result<usize> {
        let rbuf: Vec<u8> = Vec::from(buf);
        let res = self.interface.bulk_out(self.out_ep, rbuf).await.into_result()?;
        return Ok(res.actual_length());
    }

    async fn bulkrd(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rbuf = RequestBuffer::new(buf.len());
        let res = self.interface.bulk_in(self.in_ep, rbuf).await.into_result()?;
        let bytes_read = res.len();
        let dest_slice = &mut buf[..bytes_read];
        dest_slice.copy_from_slice(&res);

        return Ok(bytes_read);
    }

    pub async fn set_baud_rate(&self, baud: u32) -> io::Result<()> {
        info!("Setting baudrate to {}", baud);

        info!("Sending baudrate config");
        self.interface.control_out(ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Interface,
            request: REQCODE_SET_BAUDRATE,
            value: 0,
            index: self.interface.interface_number() as u16,
            data: &baud.to_le_bytes()
        }).await.into_result()?;
        info!("Baudrate config sent");

        return Ok(());
    }
}

#[pin_project::pin_project]
pub struct SIAndroidCom {
    inner: Arc<Mutex<InnerSIAndroidCom>>,
    #[pin]
    write_fut: Option<BoxFuture<'static, io::Result<usize>>>,
    #[pin]
    read_fut: Option<BoxFuture<'static, io::Result<(Vec<u8>, usize)>>>,
    read_buf: Vec<u8>,
    read_pos: usize
}

impl SIAndroidCom {
    pub async fn new() -> io::Result<Self> {
        Ok(
            Self {
                inner: Arc::new(Mutex::new(InnerSIAndroidCom::new().await?)),
                write_fut: None,
                read_fut: None,
                read_buf: Vec::new(),
                read_pos: 0
            }
        )
    }

    pub async fn set_baud_rate(&mut self, baud: u32) -> io::Result<()> {
        let guard = self.inner.lock().await;
        guard.set_baud_rate(baud).await
    }

    // i vibecoded this because idk what i was doing
    // the commit that added this comment will be just about removing the annoying comments the model generated
    fn poll_read_internal(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut this = self.as_mut().project();

            if *this.read_pos < this.read_buf.len() {
                let available = &this.read_buf[*this.read_pos..];
                let to_copy = available.len().min(buf.remaining());
                buf.put_slice(&available[..to_copy]);
                *this.read_pos += to_copy;

                if *this.read_pos == this.read_buf.len() {
                    this.read_buf.clear();
                    *this.read_pos = 0;
                }

                return Poll::Ready(Ok(()));
            }

            if this.read_fut.is_none() {
                let inner = Arc::clone(this.inner);
                let fut = async move {
                    let mut guard = inner.lock().await;
                    let mut temp_buf = vec![0u8; 64];
                    let l = guard.bulkrd(&mut temp_buf).await?;
                    temp_buf.truncate(l);
                    Ok((temp_buf, l))
                };
                this.read_fut.set(Some(Box::pin(fut)));
            }

            if let Some(fut) = this.read_fut.as_mut().as_pin_mut() {
                match fut.poll(cx) {
                    Poll::Ready(Ok(data)) => {
                        this.read_fut.set(None);
                        *this.read_buf = data.0;
                        *this.read_pos = 0;
                        continue;
                    },
                    Poll::Ready(Err(e)) => {
                        this.read_fut.set(None);
                        return Poll::Ready(Err(e));
                    },
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}

impl AsyncRead for SIAndroidCom {
    fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
        self.poll_read_internal(cx, buf)
    }
}

impl AsyncWrite for SIAndroidCom {
    fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, io::Error>> {
        let mut this = self.project();
        
        if this.write_fut.is_none() {
            let inner = Arc::clone(this.inner);
            let data = buf.to_vec();
            let fut = async move {
                let mut guard = inner.lock().await;
                guard.bulkwrt(&data).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            };
            *this.write_fut = Some(Box::pin(fut));
        }

        if let Some(fut) = this.write_fut.as_mut().as_pin_mut() {
            match fut.poll(cx) {
                Poll::Ready(Ok(len)) => {
                    this.write_fut.set(None);
                    return Poll::Ready(Ok(len));
                },
                Poll::Ready(Err(e)) => {
                    this.write_fut.set(None);
                    return Poll::Ready(Err(e));
                },
                Poll::Pending => return Poll::Pending
            }
        }

        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}


// dont ever uncomment this please
// impl AsyncRead for InnerSIAndroidCom {
//     fn poll_read(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> Poll<io::Result<()>> {
//         let this = self.get_mut();

//         // Use a loop to retry the operation after the buffer is filled.
//         loop {
//             // First, try to fulfill the read from the existing buffer.
//             if this.read_pos < this.read_buffer.len() {
//                 let remaining = &this.read_buffer[this.read_pos..];
//                 let to_copy = remaining.len().min(buf.remaining());
//                 buf.put_slice(&remaining[..to_copy]);
//                 this.read_pos += to_copy;
//                 return Poll::Ready(Ok(()));
//             }

//             // If the buffer is empty, we need to perform the async read.
//             // The future will hold a mutable borrow of `this`.
//             let fut = this.read_bulk();
//             tokio::pin!(fut); // Pin the future to the stack.

//             match fut.poll(cx) {
//                 // The async operation is complete and `this.read_buffer` is now filled.
//                 Poll::Ready(Ok(())) => {
//                     // The mutable borrow from `fut` ends here.
//                     // Instead of copying the data now, just loop back to the top.
//                     // The check `if this.read_pos < this.read_buffer.len()` will now pass.
//                     continue;
//                 }
//                 Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                 Poll::Pending => return Poll::Pending,
//             }
//         }
//     }
// }


// impl AsyncWrite for InnerSIAndroidCom {
//     fn poll_write(
//             self: std::pin::Pin<&mut Self>,
//             cx: &mut std::task::Context<'_>,
//             buf: &[u8],
//         ) -> Poll<Result<usize, io::Error>> {
//         let this = self.get_mut();

//         let fut = this.write_bulk(buf);
//         tokio::pin!(fut);

//         fut.poll(cx)
//     }

//     fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), io::Error>> {
//         Poll::Ready(Ok(()))
//     }
//     fn poll_shutdown(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), io::Error>> {
//         Poll::Ready(Ok(()))
//     }
// }