use vm::vm::Vm;
use clap::{Arg, App, crate_version, value_t};

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
                               .help("VM's memory size (default 128K)"))
                          .arg(Arg::with_name("bin")
                               .long("bin")
                               .short("b")
                               .takes_value(true)
                               .value_name("FILE")
                               .help("Specifies a file to be loaded into the VM's memory"))
                          .get_matches();
    
    let mut vm = Vm::new(value_t!(matches, "memory-size", usize).unwrap_or(131072));

    if let Some(ref bin) = matches.value_of("bin") {
        vm.load_from_file(bin).unwrap();
    }

    for _ in 1..100 {
        match vm.cycle() {
            Ok(()) =>  {
                println!("OK ip={} opcode={:02X} regs={:?} sp={}", vm.ip, vm.memory.read_u8(vm.ip).unwrap(), &vm.registers[1..15], vm.registers[15])
            },
            Err(e) => {
                println!("ERROR {:?}", e);
                break;
            },
        }
    }
}