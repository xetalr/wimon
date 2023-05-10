# Simple Wi-Fi monitor for Linux

If your device driver supports interfaces in monitor mode, you can use this tool.  
**CAP_NET_ADMIN** is required or `root` user.

* Make sure monitor mode is supported:  
  `iw list`  
  > `Wiphy phy0`  
  > `...`  
  > `Supported interface modes:`  
  >      `* monitor`  
  > `...`  
* Add a monitor interface on `phy0` device:  
  `iw phy0 interface add mon0 type monitor`
* Enable interface:  
  `ifconfig mon0 up`
* Run the program:  
  `wimon mon0`

You probably want to disable other interfaces on the device and use `iw mon0 set channel 1` to switch channels.

Example:
> `$> wimon mon0`  
> `AP STA: xx:xx:xx:xx:xx:xx, BSSID: xx:xx:xx:xx:xx:xx, SSID: "wifi", channel: 1 (2412 MHz, -50 dBm)`  
> `STA: xx:xx:xx:xx:xx:xx probe SSID: "wifi" (2412 MHz, -58 dBm)`  
> `STA: xx:xx:xx:xx:xx:xx probe SSID: "" (2412 MHz, -65 dBm)`  
