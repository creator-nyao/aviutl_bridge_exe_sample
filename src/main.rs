//エンコード変換周り
extern crate encoding;
use encoding::{Encoding, EncoderTrap};
use encoding::all::{WINDOWS_31J, UTF_8};

use std::{self, error};
use std::io::ErrorKind;
use std::io::{self, Write, BufReader, Read, BufWriter};

use std::ffi::CString;

//ログ系
#[macro_use] extern crate log;
extern crate simplelog;
// use log::*;
use simplelog::*;
use std::fs::File;

//共有メモリヘッダー
#[repr(C)]
struct share_mem_header {
    header_size: u32,
    body_size: u32,
    version: u32,
    width: u32,
    height: u32,
}

//
#[repr(C)]
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
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
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

    for i in 1..=3 {
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