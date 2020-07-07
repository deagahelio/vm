use vm::vm::Vm;
use clap::{Arg, App, crate_version, value_t};
use minifb::{Key, Window, WindowOptions, Scale};
use std::thread;
use std::sync::{Arc, Mutex, mpsc::channel};
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
    let (width, height) = (640, 360);
    let framebuffer = Arc::new(Mutex::new(vec![0u32; 32*1024*1024]));
    let mut vm = Vm::new(memory_size, Some(framebuffer.clone()));

    if let Some(ref bin) = matches.value_of("bios") {
        vm.load_boot(bin).unwrap();
    }

    if let Some(disks) = matches.values_of("disk") {
        for (i, path) in disks.enumerate() {
            vm.memory.set_disk(i, Some(fs::read(path).expect("could not read disk file")));
        }
    }

    let framebuffer_copy = framebuffer.clone();
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let mut window = Window::new("vm", width, height, WindowOptions { scale: Scale::X2, .. WindowOptions::default() }).unwrap();
        window.limit_update_rate(Some(std::time::Duration::from_micros(33300)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let framebuffer = framebuffer_copy.lock().unwrap();
            window.update_with_buffer(&framebuffer, width, height).unwrap();
        }

        sender.send(()).unwrap();
    });

    loop {
        match vm.cycle() {
            Ok(()) =>  {
                //println!("OK ip={:X} opcode={:02X} regs={:?} sp={}", vm.ip, vm.memory.read_u8(vm.ip).unwrap(), &vm.registers[1..15], vm.registers[15]);
            },
            Err(e) => {
                println!("ERROR {:?}", e);
                break;
            },
        }

        if receiver.try_recv().is_ok() {
            println!("mem:{:?}", &vm.memory.bytes[0x200..1000+0x200]);
            break;
        }
    }
}