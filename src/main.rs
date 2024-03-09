use std::{self, env};
use std::io::ErrorKind;
use std::io::{self, Write, BufReader, Read, BufWriter};

//ログ系
#[macro_use] extern crate log;
extern crate simplelog;
// use log::*;
use simplelog::*;
use std::fs::File;

//バイト列の構造体変換
use serde::{Serialize, Deserialize};

use windows::Win32::System::Memory::*;
use windows::Win32::Foundation::CloseHandle;

//共有メモリヘッダー
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
struct share_mem_header {
    header_size: u32,
    body_size: u32,
    version: u32,
    width: u32,
    height: u32,
}

//
#[repr(C)]
#[derive(Debug, Serialize, Deserialize,Copy,Clone)]
struct pixel {
    b: u8,
    g: u8,
    r: u8,
    a: u8,
}

// use std::mem;

// #[repr(C)]
// #[derive(Debug)]
// struct Test {
//     a: u8,
//     b: String,
// }

// impl Test {
//     fn get_raw_bytes_with_size<T: ?Sized>(p: &T, size: usize) -> Vec<u8> {
//         let mut buf = Vec::with_capacity(size);
//         let view = p as *const _ as *const u8;
//         for i in 0..size {
//             buf.push(unsafe {*view.offset(i as isize)});
//         }
//         buf
//     }
    
//     fn get_raw_bytes(&self) -> Vec<u8> {
//         Self::get_raw_bytes_with_size(self, mem::size_of::<Self>())
//     }
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = process();
    if let Err(error) = &result{
        error!("{}", error);
    }
    
    return result;
}

fn process() -> Result<(), Box<dyn std::error::Error>> {
    //ログ設定
    CombinedLogger::init(
        vec![
            // TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Warn, Config::default(), File::create("aviutl_bridge_exe_sample.log")?),
            WriteLogger::new(LevelFilter::Error, Config::default(), File::create("aviutl_bridge_exe_sample.log")?),
            WriteLogger::new(LevelFilter::Warn, Config::default(), File::create("aviutl_bridge_exe_sample.log")?),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("aviutl_bridge_exe_sample.log")?),
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("aviutl_bridge_exe_sample.log")?),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("aviutl_bridge_exe_sample.log")?),
        ]
    )?;

    info!("up!");

    //書き込み準備
    // let stdout = io::stdout().lock();
    let stdout: io::Stdout = io::stdout();
    let mut stdout_writer = BufWriter::new(stdout);

    //読み込み準備
    let mut stdin_reader = BufReader::new(io::stdin());

    for i in 1..=100 {
        info!("loop is {}", i);

        //読み込み
        
        //本体データサイズ　読み込み
        let mut read_bytes :[u8; 4] = [0; 4];
        let readable_length;
        let use_size;
        match stdin_reader.read(&mut read_bytes) {
            Ok(tmp_use_size) => {
                info!("communication start!");
                readable_length = i32::from_le_bytes(read_bytes);
                use_size = tmp_use_size;
            },
            Err(error) => {
                error!("{}", &error);
                return Err(Box::new(error));
            },
        }
        info!("input data size: {}", &readable_length);

        if 1024 <= readable_length {
            let error: io::Error = std::io::Error::new(ErrorKind::InvalidData, format!("input data too big: {}", &readable_length));
            error!("{}", &error);
            return Err(Box::new(error));
        }
        
        //本体データ読み込み
        let mut buffer = vec![0; readable_length as usize];
        if 0 < use_size {
            match stdin_reader.read(&mut buffer) {
                Ok(use_size) => {
                    if use_size == 0 {
                        let error = std::io::Error::new(ErrorKind::InvalidData, "cannot read input data");
                        error!("{}", &error);
                        return Err(Box::new(error));
                    }
                },
                Err(error) => {
                    error!("{}", &error);
                    return Err(Box::new(error));
                },
            }
        }
        info!("input data: {:?}", buffer);
        // let test = Test{
        //     a: 32,
        //     b: String::from("_test"),
        // };
        // info!("{:?}", test.get_raw_bytes());
        info!("input data: {}", std::str::from_utf8(&buffer)?);
        info!("input success!");

        //ファイルマッピングオブジェクトの読み込み・書き込み
        match env::var("BRIDGE_FMO") {
            Ok(mut fmo_name) => {
                info!("fmo_name: {}", &fmo_name);
                info!("FileMappingObject process start.");

                unsafe{
                    let mut fmo_any  = None;
                    for _ in 1..=100{
                        //ファイルマッピングオブジェクト　オープン
                        match OpenFileMappingA(
                            FILE_MAP_ALL_ACCESS.0,
                            false,
                            windows::core::PCSTR{0: fmo_name.as_ptr() as *const u8},
                        ) {
                            Ok(handle) => {
                                fmo_any = Some(handle);
                                break;
                            },
                            Err(error) => {
                                error!("{}", error);
                            }
                        };

                        //ファイル名　再読み込み
                        info!("ファイルマッピングオブジェクトが開けない為、ファイル名を読み込み直します。");
                        match env::var("BRIDGE_FMO") {
                            Ok(loaded_fmo_name) =>{
                                fmo_name = loaded_fmo_name;
                                info!("fmo_name:{}", &fmo_name);
                            },
                            Err(error) => {
                                error!("{}", &error);
                                error!("ファイル名の読み込みに失敗しました。");
                            }
                        }
                        let stop_time = std::time::Duration::from_millis(100);
                        std::thread::sleep(stop_time);
                    }
                    let fmo = match fmo_any {
                        Some(handle) => {
                            handle
                        },
                        None => {
                            let error_text = "100回試行しましたが、FileMappingObjectのファイル名が見つかりませんでした。";
                            error!("{}", &error_text);
                            let error = std::io::Error::new(ErrorKind::InvalidData, error_text);
                            return Err(Box::new(error));
                        }
                    };

                    info!("fmo open success!");

                    let view_address = MapViewOfFile(
                        fmo,
                        FILE_MAP_WRITE,
                        0,
                        0,
                        0,
                    );

                    info!("fmo mapping success!");

                    let first_pointer = view_address.Value;
                    let header = first_pointer as *mut share_mem_header;
                    info!("header:{:?}", *header);
                    let mut pixel_pointer = first_pointer.byte_offset((*header).header_size as isize) as *mut pixel;
                    info!("first_pixel:{:?}", *pixel_pointer);

                    let height = (*header).height;
                    let width = (*header).width;

                    info!("height,width:{},{}", height, width);

                    for y in 0..height{
                        for x in 0..width{
                            let mut pixel = (*pixel_pointer) as pixel;
                            
                            let alpha_f64 = pixel.a as f64;
                            pixel.a = (alpha_f64 * (y as f64) / (height as f64)) as u8;

                            let pixel = &pixel as *const pixel;
                            pixel_pointer.copy_from_nonoverlapping(pixel, 1);

                            if y <= height - 1  && x <= width - 2{
                                //最後のループ以外
                                pixel_pointer = pixel_pointer.offset(1);
                            }
                        }
                    }

                    UnmapViewOfFile(view_address)?;
                    CloseHandle(fmo)?;

                }
        
                info!("FileMappingObject process end.");

            },
            Err(error) => {
                info!("{}", &error);
                info!("ファイルマッピングオブジェクトはないものとして処理を継続します。");
            }
        }




        //書き込み
        let out_message = format!("Hello world {}", i);
        let out_message_size:i32 = out_message.clone().len() as i32;
        
        info!("output data size: {}", &out_message_size);
        info!("output data: {}", &out_message);

        //本体データサイズ　書き込み
        match stdout_writer.write(&out_message_size.to_le_bytes()) {
            Ok(_) => {},
            Err(error) => {
                error!("{}", &error);
                return Err(Box::new(error));
            }
        }
        

        //本体データ　書き込み
        let out_message_bytes ;
        out_message_bytes = out_message.clone().into_bytes();
        match stdout_writer.write(&out_message_bytes) {
            Ok(_) => {
                info!("output success!");
            },
            Err(error) => {
                error!("{}", &error);
                return Err(Box::new(error));
            },
        }

        match stdout_writer.flush() {
            Ok(_) => {
                info!("beffer flush!");
            },
            Err(error) => {
                error!("{}", &error);
                return Err(Box::new(error));
            },
            
        }

        info!("communication end!");
    }
    
    info!("down!");
    Ok(())
}