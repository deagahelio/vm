use vm::vm::Vm;
use clap::{Arg, App, crate_version, value_t};
use minifb::{Key, Window, WindowOptions};

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
                          .arg(Arg::with_name("bin")
                               .long("bin")
                               .short("b")
                               .takes_value(true)
                               .value_name("FILE")
                               .help("Specifies a file to be loaded into the VM's memory"))
                          .get_matches();
    
    let mut vm = Vm::new(value_t!(matches, "memory-size", usize).unwrap_or(134217728));

    if let Some(ref bin) = matches.value_of("bin") {
        vm.load_from_file(bin).unwrap();
    }

    let (width, height) = (640, 360);
    let mut buffer: Vec<u32> = vec![0; width * height];
    let mut window = Window::new("vm", width, height, WindowOptions::default()).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (i, pixel) in buffer.iter_mut().enumerate() {
            *pixel = (vm.memory.bytes[0x100000 + i * 3] as u32) << 16 |
                     (vm.memory.bytes[0x100001 + i * 3] as u32) << 8 |
                     (vm.memory.bytes[0x100002 + i * 3] as u32);
        }

        match vm.cycle() {
            Ok(()) =>  {
                println!("OK ip={} opcode={:02X} regs={:?} sp={}", vm.ip, vm.memory.read_u8(vm.ip).unwrap(), &vm.registers[1..15], vm.registers[15])
            },
            Err(e) => {
                println!("ERROR {:?}", e);
                break;
            },
        }

        window.update_with_buffer(&buffer, width, height).unwrap();
    }

    println!("mem:{:?}", &vm.memory.bytes[..1000]);
}