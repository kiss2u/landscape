// use sysinfo::{Components, Disks, Networks, System};
use sysinfo::System;

use crate::{LandscapeStatic, LandscapeStatus};

pub fn init_base_info() -> (LandscapeStatic, LandscapeStatus) {
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());
    let l_static = LandscapeStatic {
        name: System::name().unwrap_or("None".to_string()),
        kernel_version: System::kernel_version().unwrap_or("None".to_string()),
        os_version: System::os_version().unwrap_or("None".to_string()),
        host_name: System::host_name().unwrap_or("None".to_string()),
    };
    (l_static, LandscapeStatus::new())
}
