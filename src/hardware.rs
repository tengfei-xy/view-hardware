use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Cpu {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "NumberOfCores")]
    number_of_cores: u32,
    #[serde(rename = "NumberOfLogicalProcessors")]
    number_of_logical_processors: u32,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OptCPU {
    StructType(Cpu),
    ArrayType(Vec<Cpu>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Memory {
    #[serde(rename = "Capacity")]
    capacity: u64,
    #[serde(rename = "Speed")]
    speed: u32,
    #[serde(rename = "MemoryType")]
    memory_type_seq: u32,
    #[serde(skip)]
    memory_type: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OptMemory {
    StructType(Memory),
    ArrayType(Vec<Memory>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Disk {
    #[serde(rename = "MediaType")]
    media_type: String,
    #[serde(rename = "FriendlyName")]
    friendly_name: String,
    #[serde(rename = "Size")]
    size: u64,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OptDisk {
    StructType(Disk),
    ArrayType(Vec<Disk>),
}

pub struct Win {
    c: OptCPU,
    m: OptMemory,
    d: OptDisk,
}
pub struct Hardware {
    c: Vec<Cpu>,
    m: Vec<Memory>,
    d: Vec<Disk>,
}
#[allow(dead_code)]
pub struct Mac {
    c: Vec<Cpu>,
    m: Vec<Memory>,
    d: Vec<Disk>,
}
impl Hardware {
    pub fn build() -> Hardware {
        #[cfg(target_os = "windows")]
        {
            Win::total()
        }
        #[cfg(target_os = "macos")]
        {
            Mac::build()
        }
        #[cfg(target_os = "linux")]
        {
            // Linux::build()
        }
    }
    fn check_cpu_same(&self) -> bool {
        if self.c.len() == 1 {
            return true;
        }
        let mut name = String::new();
        for c in &self.c {
            if name.is_empty() {
                name = c.name.clone();
            }
            if name != c.name {
                return false;
            }
        }
        true
    }
    pub fn get_cpu(&self) -> String {
        if self.check_cpu_same() {
            return format!(
                "{} * {},{}核{}线程",
                self.c.len(),
                self.c[0].name,
                self.c[0].number_of_cores,
                self.c[0].number_of_logical_processors
            );
        }
        let mut s = String::new();
        for c in &self.c {
            s.push_str(
                format!(
                    "{} {}核{}线程,",
                    c.name, c.number_of_cores, c.number_of_logical_processors
                )
                .as_str(),
            );
        }
        s
    }
    fn check_memory_same(&self) -> bool {
        if self.m.len() == 1 {
            return true;
        }
        let mut capacity = 0;
        let mut speed = 0;
        for m in &self.m {
            if capacity == 0 {
                capacity = m.capacity;
                speed = m.speed;
            }
            if capacity != m.capacity || speed != m.speed {
                return false;
            }
        }
        true
    }
    pub fn get_memory(&self) -> String {
        if self.check_memory_same() {
            return format!(
                "{} * {}GB,{}MHz,{}",
                self.m.len(),
                self.m[0].capacity,
                self.m[0].speed,
                self.m[0].memory_type
            );
        }


        let mut s = String::new();
        for m in &self.m {
            s.push_str(format!("{}GB {}MHz {},", m.capacity, m.speed, m.memory_type).as_str());
        }

        s
    }
    fn check_disk_same(&self) -> bool {
        if self.d.len() == 1 {
            return true;
        }

        let mut media_type = String::new();
        let mut size = 0;
        for d in &self.d {
            if media_type.is_empty() {
                media_type = d.media_type.clone();
                size = d.size;
            }
            if media_type != d.media_type || size != d.size {
                return false;
            }
        }
        true
    }
    pub fn get_disk(&self) -> String {
        if self.check_disk_same() {
            return format!(
                "{} * {}GB,{}",
                self.d.len(),
                self.d[0].size,
                self.d[0].media_type
            );
        }

        let mut s = String::new();
        for d in &self.d {
            s.push_str(format!("{}GB {},", d.size, d.media_type).as_str());
        }
        s
    }

    pub fn display(&self) {
        println!("CPU: {}", self.get_cpu());
        println!("Memory: {}", self.get_memory());
        println!("Disk: {}", self.get_disk());
    }
}

#[allow(dead_code)]
impl Win {
    pub fn build() -> Win {
        // 创建一个通道，用于从线程中返回结果
        let (cpu_t, cpu_r) = mpsc::channel();
        let (memory_t, memory_x) = mpsc::channel();
        let (disk_t, disk_x) = mpsc::channel();

        // 启动第一个函数的线程
        let tx1 = cpu_t.clone();
        thread::spawn(move || {
            let result = Win::get_cpu();
            tx1.send(result).unwrap();
        });

        // 启动第二个函数的线程
        let tx2 = memory_t.clone();
        thread::spawn(move || {
            let result = Win::get_memory();

            tx2.send(result).unwrap();
        });

        // 启动第三个函数的线程
        let tx3 = disk_t.clone();
        thread::spawn(move || {
            let result = Win::get_disk();
            tx3.send(result).unwrap();
        });

        // 等待三个通道中的结果，并打印它们
        Win {
            c: cpu_r.recv().unwrap(),
            m: memory_x.recv().unwrap(),
            d: disk_x.recv().unwrap(),
        }
    }
    fn powershell(cmd: &str) -> String {
        let cmd_result = Command::new("powershell")
            .args(&["-Command", cmd])
            .output()
            .expect("failed to execute process");

        String::from_utf8_lossy(&cmd_result.stdout).to_string()
    }
    fn get_cpu() -> OptCPU {
        let cmd_result = Win::powershell("Get-WmiObject -Class Win32_Processor | select Name,NumberOfCores,NumberOfLogicalProcessors | ConvertTo-Json");
        serde_json::from_str(cmd_result.as_str()).unwrap()
    }
    pub fn convert_cpu(&self) -> Vec<Cpu> {
        let mut v: Vec<Cpu> = Vec::new();
        match &self.c {
            OptCPU::StructType(cpu) => v.push(cpu.clone()),
            OptCPU::ArrayType(cpu_vec) => v = (*cpu_vec.clone()).to_vec(),
        }
        v
    }

    fn get_memory() -> OptMemory {
        let cmd_result = Win::powershell("Get-WmiObject -Class Win32_PhysicalMemory | Select-Object @{Name='Capacity'; Expression={$_.Capacity / 1GB}}, Speed, MemoryType | ConvertTo-Json");
        let mut s = serde_json::from_str(cmd_result.as_str()).unwrap();
        match &mut s {
            OptMemory::StructType(memory) => {
                memory.memory_type = Win::get_memory_type(memory.memory_type_seq).to_string();
            }
            OptMemory::ArrayType(memory_vec) => {
                for memory in memory_vec {
                    memory.memory_type =  Win::get_memory_type(memory.memory_type_seq).to_string();
                }
            }
        }
        s
    }

    pub fn convert_memory(&self) -> Vec<Memory> {
        let mut v: Vec<Memory> = Vec::new();
        match &self.m {
            OptMemory::StructType(memory) => v.push(memory.clone()),
            OptMemory::ArrayType(memory_vec) => v = (*memory_vec.clone()).to_vec(),
        }
        v
    }
    fn get_disk() -> OptDisk {
        let cmd_result = Win::powershell(
            "Get-PhysicalDisk |select mediaType,FriendlyName,Size | ConvertTo-Json",
        );
        let mut s = serde_json::from_str(cmd_result.as_str()).unwrap();
        match &mut s {
            OptDisk::StructType(disk) => {
                disk.size = Win::rounding(disk.size);
            }
            OptDisk::ArrayType(disk_vec) => {
                for disk in disk_vec {
                    disk.size = Win::rounding(disk.size);
                }
            }
        }
        s

    }
    pub fn convert_disk(&self) -> Vec<Disk> {
        let mut v: Vec<Disk> = Vec::new();
        match &self.d {
            OptDisk::StructType(disk) => v.push(disk.clone()),
            OptDisk::ArrayType(disk_vec) => v = (*disk_vec.clone()).to_vec(),
        }
        v
    }
    fn get_memory_type(memory_type_seq: u32) -> &'static str {
        match memory_type_seq {
            20 => "DDR",
            21 => "DDR2",
            22 => "DDR2 FB-DIMM",
            23 => "DDR3",
            24 => "DDR3",
            25 => "FBD2",
            26 => "DDR4",
            _ => "未知",
        }
    }
    fn rounding(mut d: u64) -> u64 {
        d = d / 1024 / 1024 / 1024;
        let v: [u64; 6] = [120, 240, 500, 1000, 2000, 4000];
        let mut min_diff = std::u64::MAX;
        let mut closest_value: u64 = d;

        for &n in v.iter() {
            let diff = (d as i64 - n as i64).abs() as u64;
            if diff < min_diff {
                min_diff = diff;
                closest_value = n;
            }
        }

        closest_value
    }
    fn total() -> Hardware {
        let os = Win::build();
        let c = os.convert_cpu();
        let m = os.convert_memory();
        let d = os.convert_disk();
        Hardware { c, m, d }
    }
}
impl Mac {
    fn build() -> Hardware {
        let c = Mac::get_cpu();
        let m = Mac::get_memory();
        let d = Mac::get_disk();
        Hardware { c, m, d }
    }
    fn command(cmd: &str) -> String {
        let cmd_result = Command::new("sh")
            .args(&["-c", cmd])
            .output()
            .expect("failed to execute process");

        String::from_utf8_lossy(&cmd_result.stdout).to_string()
    }
    fn get_cpu() -> Vec<Cpu> {
        let mut retval: Vec<Cpu> = Vec::with_capacity(1);

        let name = Mac::command("sysctl -n machdep.cpu.brand_string");

        let core_count = Mac::command("sysctl -n machdep.cpu.core_count");

        let thread_count = Mac::command("sysctl -n machdep.cpu.thread_count");

        retval.push(Cpu {
            name: name.trim().to_string(),
            number_of_cores: core_count.trim().parse::<u32>().unwrap(),
            number_of_logical_processors: thread_count.trim().parse::<u32>().unwrap(),
        });
        retval
    }
    fn get_memory() -> Vec<Memory> {
        let cmd_result = Mac::command("system_profiler SPMemoryDataType");

        let mut retval: Vec<Memory> = Vec::new();

        let lines = cmd_result.trim().lines().map(|line| line.trim());
        let mut iter = lines.into_iter().peekable();

        while let Some(line) = iter.next() {
            if line.starts_with("Size:") {
                let capacity: u64 = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or_default();
                let memory_type = iter
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_string();
                let speed: u32 = iter
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or_default();

                retval.push(Memory {
                    capacity,
                    speed,
                    memory_type_seq: 1,
                    memory_type,
                });
            }
        }
        retval
    }
    fn get_disk() -> Vec<Disk> {
        let mut retval: Vec<Disk> = Vec::new();
        let cmd = Mac::command("diskutil info -plist /dev/disk0 | plutil -convert json -o - -");
        let value: Value = serde_json::from_str(cmd.as_str()).unwrap();
        let media_name: String = value.get("MediaName").unwrap().to_string();
        let media_type = if media_name.contains("SSD") {
            "SSD"
        } else {
            "未知"
        };
        let size = value.get("Size").unwrap().as_u64().expect("未知");

        retval.push(Disk {
            friendly_name: media_name,
            media_type: media_type.to_string(),
            size: Mac::rounding(size) ,
        });
        retval
    }
    fn rounding(mut d : u64) -> u64 {
        d = d /1024/1024/1024;
        let v: [u64; 6] = [120, 240, 500, 1000, 2000, 4000];
        let mut min_diff = std::u64::MAX;
        let mut closest_value: u64 = d;

        for &n in v.iter() {
            let diff = (d as i64 - n as i64).abs() as u64;
            if diff < min_diff {
                min_diff = diff;
                closest_value = n;
            }
        }
        closest_value
    }
}
