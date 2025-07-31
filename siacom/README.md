# SPORTident Android Communication Library

⚠️ **THIS CRATE COMPILES ONLY FOR ANDROID TARGETS** ⚠️  
*That means you should add `siacom` as a dependency **only** when compiling for Android (`target.'cfg(target_os = "android")'.dependencies`).*

⚠️ This library exposes **only the async** read/write API by implementing Tokio's `AsyncRead` and `AsyncWrite`. ⚠️

⚠️ **THIS LIBRARY IS MADE EXCLUSIVELY FOR COMMUNICATING WITH SPORTident DEVICES.**  
It actively filters by **VID** and **PID**, and will **not work** with general CP2102-based devices. ⚠️

## Overview

This library enables communication with `SPORTident USB to UART` devices on Android.  
*Note: This is **not** an official implementation by SPORTident.*

## Requirements

Before using this library, a few setup steps are required:

1. **Add permissions** (`AndroidManifest.xml`):  
   - Add:  
     ```xml
     <uses-feature android:name="android.hardware.usb.host" />  
     <uses-permission android:name="android.permission.USB_PERMISSION" />
     ```

2. **Add intent filter** (`AndroidManifest.xml`, `res/xml/device_filter.xml`):  
   - Add the following `<intent-filter>` to your activity:  
     ```xml
     <intent-filter>
       <action android:name="android.hardware.usb.action.USB_DEVICE_ATTACHED" />
     </intent-filter>
     ```
   - Add this right after the intent filter:  
     ```xml
     <meta-data
       android:name="android.hardware.usb.action.USB_DEVICE_ATTACHED"
       android:resource="@xml/device_filter" />
     ```
   - If you don’t have a `res/xml/device_filter.xml` file, create it and paste [this content](https://pastebin.com/zVCMMg3e) inside.  
   - In that file, include this entry to match SPORTident devices:  
     ```xml
     <usb-device vendor-id="0x10C4" product-id="0x800A" />
     ```

## Technical Notes

When you call `SIAndroidComm::new`, the library automatically requests permission using `android.hardware.usb.UsbManager.requestPermission`.  
Currently, the result of this request is **not checked**, which may lead to unexpected behavior.  
This issue will be addressed in a future update.
