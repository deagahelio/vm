use vm::vm::Vm;
use clap::{Arg, App, crate_version, value_t};
use std::fs;

fn main() {
    let matches = App::new("vm")
                          .version(crate_version!())
                          .author("deagahelio")
                          .about("Virtual machine")
                          .arg(Arg::with_name("memory-size")
                               .long("memory-size")
                               .short("m")
                               .takes_value(true)
                               .value_name("SIZE")
                               .help("VM's memory size (default 128M)"))
                          .arg(Arg::with_name("boot")
                               .long("boot")
                               .short("b")
                               .takes_value(true)
                               .value_name("FILE")
                               .help("Specifies boot firmware to be loaded"))
                          .arg(Arg::with_name("disk")
                               .long("disk")
                               .short("d")
                               .takes_value(true)
                               .value_name("FILE")
                               .multiple(true)
                               .max_values(8)
                               .help("Specifies a file to be loaded as disk (max 8 disks)"))
                          .get_matches();
    
    let memory_size = value_t!(matches, "memory-size", usize).unwrap_or(134217728);

    let mut vm = Vm::new(memory_size);

    if let Some(ref bin) = matches.value_of("boot") {
        vm.memory.borrow_mut().load_boot(bin).unwrap();
    }

    if let Some(disks) = matches.values_of("disk") {
        let mut disk_controller = vm.disk_controller.borrow_mut();
        for (i, path) in disks.enumerate() {
            disk_controller.set_disk(i, Some(fs::read(path).expect("could not read disk file")));
        }
    }

    vm.run();
}