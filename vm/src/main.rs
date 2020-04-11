use vm::vm::Vm;
use clap::{Arg, App, crate_version, value_t};
use minifb::{Key, Window, WindowOptions};
use std::thread;
use std::sync::{Arc, Mutex, mpsc::channel};

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
    
    let memory_size = value_t!(matches, "memory-size", usize).unwrap_or(134217728);
    let vm = Arc::new(Mutex::new(Vm::new(memory_size)));

    if let Some(ref bin) = matches.value_of("bin") {
        vm.lock().unwrap().load_from_file(bin).unwrap();
    }

    let (width, height) = (640, 360);

    let vm_copy = vm.clone();
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let mut window = Window::new("vm", width, height, WindowOptions::default()).unwrap();
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let vm = vm_copy.lock().unwrap();
            window.update_with_buffer(&vm.memory.framebuffer, width, height).unwrap();
        }

        println!("mem:{:?}", &vm_copy.lock().unwrap().memory.bytes[..1000]);
        sender.send(()).unwrap();
    });

    loop {
        let mut vm = vm.lock().unwrap();
        match vm.cycle() {
            Ok(()) =>  {
                println!("OK ip={} opcode={:02X} regs={:?} sp={}", vm.ip, vm.memory.read_u8(vm.ip).unwrap(), &vm.registers[1..15], vm.registers[15]);
            },
            Err(e) => {
                println!("ERROR {:?}", e);
                break;
            },
        }

        if receiver.try_recv().is_ok() {
            break;
        }
    }
}