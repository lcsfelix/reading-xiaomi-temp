use std::thread;
use std::time::Duration;
use blurz::{
    BluetoothGATTCharacteristic,
    BluetoothEvent,
    BluetoothSession,
    BluetoothAdapter,
    BluetoothDiscoverySession,
    BluetoothDevice
};

mod explore_device;

fn main() {
    let bt_session = &BluetoothSession::create_session(None).unwrap();
    let adapter: BluetoothAdapter = BluetoothAdapter::init(bt_session).unwrap();
    if let Err(_error) = adapter.set_powered(true) {
        panic!("Failed to power adapter");
    }

    let discover_session = BluetoothDiscoverySession::create_session(&bt_session, adapter.get_id()).unwrap();
    if let Err(_error) = discover_session.start_discovery() {
        panic!("Failed to start discovery");
    }
    let device_list = adapter.get_device_list().unwrap();

    discover_session.stop_discovery().unwrap();

    println!("{:?} devices found", device_list.len());

    for device_path in device_list {
        let device = BluetoothDevice::new(bt_session, device_path.to_string());
        println!("Device: {:?} Name: {:?}", device_path, device.get_name().ok());
    }

    println!();
    
    let device = BluetoothDevice::new(bt_session, String::from("/org/bluez/hci0/dev_A4_C1_38_64_7E_DB"));
    // let device = BluetoothDevice::new(bt_session, String::from("/org/bluez/hci0/dev_A4_C1_38_15_03_55"));

    if let Err(e) = device.connect(10000) {
        println!("Failed to connect {:?}: {:?}", device.get_id(), e);
    } else {
        // We need to wait a bit after calling connect to safely
        // get the gatt services
        thread::sleep(Duration::from_millis(5000));
        explore_device::explore_gatt_profile(bt_session, &device);
        let temp_humidity = BluetoothGATTCharacteristic::new(bt_session, device.get_id() + "/service0021/char0035");
        temp_humidity.start_notify().unwrap();

        println!("READINGS");
        loop {
            for event in BluetoothSession::create_session(None).unwrap().incoming(1000).map(BluetoothEvent::from) {
                if event.is_none() {
                    continue;
                }

                let value = match event.clone().unwrap() {
                    BluetoothEvent::Value {object_path : _, value} => value,
                    _ => continue
                };

                let mut temperature_array = [0; 2];
                temperature_array.clone_from_slice(&value[..2]);
                let temperature = u16::from_le_bytes(temperature_array) as f32 * 0.01;
                let humidity = value[2];
                println!("Temperature: {:.2}ºC Humidity: {:?}%", temperature, humidity);
            }
        }
    }
}