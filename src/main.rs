use std::{
    self,
    fs::{self, File},
    io::Read,
    path::Path,
    process, thread,
    time::Duration,
};

fn get_module_base(pid: u32,name:&str) ->usize{
        for maps in get_process_maps(pid){
        for line in maps{
            if line.filename().as_deref().unwrap_or("") ==name && line.is_read()&&line.is_write()&&!line.is_exec(){
                return line.start();
            }
        }
    }
    0
}

fn findpid(name: &str) -> u32 {
    let mut pid: u32 = 0;
    for process in fs::read_dir("/proc").unwrap() {
        let comm = format!("{}/comm", process.unwrap().path().display());
        let file = File::open(Path::new(&comm));
        if let Ok(mut f) = file {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            if s.trim() == name {
                let split: Vec<&str> = comm.split("/").collect();
                pid = split[2].parse().unwrap();
                break;
            }
        }
    }
    pid
}

#[derive(Debug, Clone, PartialEq)]
struct MapRange {
    range_start: usize,
    range_end: usize,
    offset: usize,
    dev: String,
    flags: String,
    inode: usize,
    pathname: Option<String>,
}

impl MapRange {
    fn size(&self) -> usize {
        self.range_end - self.range_start
    }
    fn start(&self) -> usize {
        self.range_start
    }
    fn filename(&self) -> &Option<String> {
        &self.pathname
    }
    fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }
    fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }
    fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }
}

fn get_process_maps(pid: u32) -> std::io::Result<Vec<MapRange>> {
    let maps_file = format!("/proc/{}/maps", pid);
    let mut file = File::open(maps_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(parse_proc_maps(&contents))
}

fn parse_proc_maps(contents: &str) -> Vec<MapRange> {
    let mut vec: Vec<MapRange> = Vec::new();
    for line in contents.split("\n") {
        let mut split = line.split_whitespace();
        let range = split.next();
        if range == None {
            break;
        }
        let mut range_split = range.unwrap().split("-");
        let range_start = range_split.next().unwrap();
        let range_end = range_split.next().unwrap();
        let flags = split.next().unwrap();
        let offset = split.next().unwrap();
        let dev = split.next().unwrap();
        let inode = split.next().unwrap();

        vec.push(MapRange {
            range_start: usize::from_str_radix(range_start, 16).unwrap(),
            range_end: usize::from_str_radix(range_end, 16).unwrap(),
            offset: usize::from_str_radix(offset, 16).unwrap(),
            dev: dev.to_string(),
            flags: flags.to_string(),
            inode: usize::from_str_radix(inode, 10).unwrap(),
            pathname: Some(split.collect::<Vec<&str>>().join(" ")).filter(|x| !x.is_empty()),
        });
    }
    vec
}

fn main() {

    let pid = findpid("hello");

    thread::spawn(move || {
        let a =get_module_base(pid,"[heap]");
        if a==0{
            process::exit(0);
        }
        loop {
            thread::sleep(Duration::from_millis(1000));
            println!("{:x}",a);
        }        
    });

    loop {
        thread::sleep(Duration::from_millis(1000));
        if findpid("hello") == 0 {
            process::exit(0);
        }
    }
}


fn open_mem(pid: u32) -> File {
    let path = format!("/proc/{}/mem", pid);
    let file = File::open(&Path::new(&path)).expect("aaaa");
    return file;
}
