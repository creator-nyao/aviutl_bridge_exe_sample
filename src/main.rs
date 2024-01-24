extern crate encoding;
// extern crate winapi;

//エンコード変換周り
use encoding::{Encoding, EncoderTrap};
use encoding::all::{WINDOWS_31J, UTF_8};

use std::{self, error, env};
use std::io::ErrorKind;
use std::io::{self, Write, BufReader, Read, BufWriter};

use std::ffi::CString;

//ファイルマッピングオブジェクト系
// use std::ptr;
// use winapi::um::winbase::OpenFileMappingA;
// use winapi::um::memoryapi::MapViewOfFile;
// use winapi::um::handleapi::CloseHandle;
use file_mmap::FileMmap;

//ログ系
#[macro_use] extern crate log;
extern crate simplelog;
// use log::*;
use simplelog::*;
use std::fs::File;

//バイト列の構造体変換
use bincode;
// use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};

use windows::Win32::System::Memory::*;
use windows::Win32::Foundation::CloseHandle;
// use core_foundation::base;

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
#[derive(Debug, Serialize, Deserialize)]
struct pixel {
    b: u8,
    g: u8,
    r: u8,
    a: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //ログ設定
    CombinedLogger::init(
        vec![
            // TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Warn, Config::default(), File::create("aviutl_bridge_exe_sample.log").unwrap()),
            WriteLogger::new(LevelFilter::Error, Config::default(), File::create("aviutl_bridge_exe_sample.log").unwrap()),
            WriteLogger::new(LevelFilter::Warn, Config::default(), File::create("aviutl_bridge_exe_sample.log").unwrap()),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("aviutl_bridge_exe_sample.log").unwrap()),
            WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("aviutl_bridge_exe_sample.log").unwrap()),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("aviutl_bridge_exe_sample.log").unwrap()),
        ]
    ).unwrap();

    info!("up!");

    //書き込み準備
    // let stdout = io::stdout().lock();
    let stdout: io::Stdout = io::stdout();
    let mut stdout_writer = BufWriter::new(stdout);

    //読み込み準備
    let mut stdin_reader = BufReader::new(io::stdin());

    // refferecne to https://qiita.com/fujitayy/items/12a80560a356607da637

    for i in 1..=100 {
        info!("loop is {}", i);

        // refference to https://keens.github.io/blog/2016/12/01/rustdebaitoretsuwoatsukautokinotips/

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
        info!("input data: {}", std::str::from_utf8(&buffer).unwrap());
        info!("input success!");

        // {
        //     debug!("binary convert test start!");
        //     let test = share_mem_header{
        //         header_size: 2,
        //         body_size: 4,
        //         version: 8,
        //         width: 16,
        //         height: 32,
        //     };

        //     let test_bin = bincode::serialize(&test)?;
        //     let out_test:share_mem_header = bincode::deserialize(&test_bin[..])?;
        //     debug!("{:#?}", out_test);
        //     debug!("binary convert test end!");
            
        // }



        //ファイルマッピングオブジェクトの読み込み・書き込み
        match env::var("BRIDGE_FMO") {
            Ok(fmo_name) => {
                info!("fmo_name: {}", &fmo_name);
                info!("FileMappingObject process start.");


// let mut debug = vec![0; readable_length as usize];
// stdin_reader.read(&mut debug).unwrap();
                // unsafe{
                //     let handle = OpenFileMappingA(
                //         winapi::um::winnt::FILE_ALL_ACCESS,
                //         winapi::shared::minwindef::FALSE,
                //         fmo_name.as_ptr() as *const i8,
                //     );
            
                //     if handle.is_null() {
                //         let error: io::Error = std::io::Error::new(
                //             ErrorKind::InvalidData,
                //             format!("Failed to open file mapping object"),
                //         );
                //         error!("{}", &error);
                //         return Err(Box::new(error));
                //     } else {
                //         info!("File mapping object opened successfully!");
                //         // handle を使ってメモリマッピングオブジェクトを操作する
                //         // ...
                //     }
                // }

                unsafe{
                    let fmo = OpenFileMappingA(
                        FILE_MAP_ALL_ACCESS.0,
                        false,
                        windows::core::PCSTR{0: fmo_name.as_ptr() as *const u8},
                    ).unwrap();

                    info!("fmo open success!");

                    let view_address = MapViewOfFile(
                        fmo,
                        FILE_MAP_WRITE,
                        0,
                        0,
                        0,
                    );

                    info!("fmo mapping success!");

                    // header = *(view_address.Value) as *const share_mem_header;

                    // let header: *mut share_mem_header<[u8]> = unsafe {
                    //     std::ptr::slice_from_raw_parts_mut(view_address.Value as *mut u8, 1)
                    // } as _;

                    // let bytes = unsafe{
                    //     core::slice::from_raw_parts_mut(view_address.Value as *mut u8, 1)
                    // } as &mut[u8];

                    // let bytes = core::slice::from_raw_parts_mut(view_address.Value as *mut u8, 1) as &mut[u8];
                    // header = bincode::deserialize(bytes).unwrap();

                    let first_pointer = view_address.Value;
                    let header = first_pointer as *mut share_mem_header;
                    info!("header:{:?}", *header);
                    let mut pixel = first_pointer.byte_offset((*header).header_size as isize) as *mut pixel;
                    info!("first_pixel:{:?}", *pixel);

                    let height = (*header).height;
                    let width = (*header).width;

                    info!("height,width:{},{}", height, width);

                    for y in 0..height{
                        for x in 0..width{
                            // debug!("raito:{}", (y as f64) / (height as f64));
                            // let mut a_sub = (*pixel).a;
                            // info!("before-pixel:x={},y={}:{:?}", x, y, *pixel);
                            let alpha_f64 = (*pixel).a as f64;
                            (*pixel).a = (alpha_f64 * (y as f64) / (height as f64)) as u8;
                            // a_sub -= (*pixel).a;
                            // info!("after-pixel:x={},y={}:{:?}", x, y, *pixel);
                            // info!("after-pixel:x={},y={}:{:?}", x, y, *pixel);

                            if y <= height - 1  && x <= width - 2{
                                //最後のループ以外
                                pixel = pixel.offset(1);
                            }
                            // debug!("sub-a[{},{}]: {}",x, y, a_sub);
                        }
                    }

                    // let header:share_mem_header;
                    // match raw_pointer_to(view_address.Value as *mut u8){
                    //     Ok(load_header) => {
                    //         header = load_header;
                    //         info!("share_mem_header mapping complete.");
                    //         info!("{:?}", header);
        
                    //     }
                    //     Err(error) => {
                    //         error!("{:?}", error);
                    //     }
                    // };


                    UnmapViewOfFile(view_address)?;
                    CloseHandle(fmo)?;


                    

                    



                    // 以下を参考に
                    // https://zenn.dev/shotaro_tsuji/scraps/d921ab8c1231e9
                    // https://doc.rust-lang.org/std/ptr/index.html
                }
        
// error!("FileMmapクレートを使用する方法を諦め、memmapクレートを使う方法を考える"); 
// debug!("FileMappingObject[{}] open.", &fmo_name);               

                

                // let mut fmo: FileMmap;
                // match FileMmap::new(&fmo_name) {
                //     Ok(tmp_fmo) => {
                //         info!("FileMappingObject[{}] open.", &fmo_name);
                //         fmo = tmp_fmo;
                        
                //         // refference to https://zenn.dev/woden/articles/4dd0e7a08d3eee
                //         // refference to https://serde.rs/derive.html

                //         // ヘッダー読み込み
                //         info!("FileMappingObject header read start.");
                //         let bytes = unsafe{ fmo.bytes(0, std::mem::size_of::<share_mem_header>()) };
                //         info!("header convert to object.");

                //         match bincode::deserialize(&bytes) {
                //             Ok(obj) => {
                //                 info!("header convert success.");
                //                 let header:share_mem_header = obj;
                                
                //                 info!("test.....");
                //                 let width = header.width;
                //                 let height = header.height;
                //                 info!("FileMappingObject header read end.");

                //                 // ピクセルデータ読み込み
                //                 info!("FileMappingObject pixel read start.");
                //                 let pixel_data_address = header.header_size as usize;
                //                 let mut loop_counter = 0;
                //                 for y in 0..header.height{
                //                     for x in 0..header.width{
                //                         info!("x:{}, y:{} pixel writing...",x, y);
                //                         let bytes = unsafe{ fmo.bytes(header.header_size as isize + loop_counter, std::mem::size_of::<pixel>())};
                //                         let mut pixel:pixel = bincode::deserialize(&bytes)?;
                //                         pixel.a = (pixel.a as f64 *( y as f64 / header.height as f64)) as u8;
                //                         let write_bytes = bincode::serialize(&pixel)?;
                //                         fmo.write(header.header_size as isize + loop_counter, &write_bytes);

                //                         loop_counter += 1;
                //                     }
                //                 }
                //                 info!("FileMappingObject pixel read end.");
                //                 info!("FileMappingObject[{}] close.", &fmo_name);
                //             },
                //             Err(error) => {
                //                 error!("{}", &error);
                //                 return Err(error);
                //             }
                //         };
                //     },
                //     Err(error) => {
                //         error!("{}", &error);
                //         return Err(Box::new(error));
                //     },
                // }
                
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

// unsafe fn raw_pointer_to<'a, T>(raw_pointer: *mut u8) -> Result<T, Box<bincode::ErrorKind>> where
//     T: Deserialize<'a>
// {
//     let bytes = core::slice::from_raw_parts_mut(raw_pointer, 1) as &mut[u8];
//     return bincode::deserialize(bytes);
// }