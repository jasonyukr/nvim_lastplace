use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::str;

fn read_uint<R: BufRead>(reader: &mut R) -> (usize, usize) {
    let mut len = [0u8];
    let mut len2 = [0u8; 2];
    let mut len4 = [0u8; 4];
    let length : usize;

    match reader.read_exact(&mut len) {
        Ok(()) => {},
        Err(_) => { panic!("read_uint fail") },
    }
    match len[0] {
        0xCC => {
            reader.read_exact(&mut len).unwrap();
            length = len[0] as usize;
            return (length, 2);
        },
        0xCD => {
            reader.read_exact(&mut len2).unwrap();
            length = ((len2[0] as usize) << 8) +
                (len2[1] as usize);
            return (length, 3);
        },
        0xCE => {
            reader.read_exact(&mut len4).unwrap();
            length = ((len4[0] as usize) << 24) +
                ((len4[1] as usize) << 16) +
                ((len4[2] as usize) << 8) +
                (len4[3] as usize);
            return (length, 5);
        },
        0xCF => {
            // uint64 case
            // I don't expect this case in real life !!
            panic!("uint64 case");
        },
        0xD0|0xD1|0xD2|0xD3 => {
            // signed int case
            panic!("signed int case");
        },
        _ => {
            length = len[0] as usize;
            if length >= 0xE0 {
                panic!("signed int case");
            }
            return (length, 1);
        },
    }
}

fn main() {
    let file = File::open("/home/jinhyu/.local/state/nvim/shada/main.shada").unwrap();
    let mut reader = BufReader::new(file);

    loop {
        let mut entry_type = [0u8; 2];
        let mut timestamp = [0u8; 4];
        let mut tag = [0u8];
        let mut key = [0u8];
        let total_length;
        let mut length;
        let mut consumed;
        let mut processed : usize;

        match reader.read_exact(&mut entry_type) {
            Ok(()) => {},
            Err(_) => { break }, // we expect EOF here
        }
        match reader.read_exact(&mut timestamp) {
            Ok(()) => {},
            Err(_) => { panic!("no timestamp") },
        }
        (total_length, _) = read_uint(&mut reader);

        if entry_type[1] == 0xCE && entry_type[0] == 0x0A { // LocalMark
            /*
               -----------------------------------------------------
               Data contained in the map:
               Key  Type      Default  Description  
               l    UInteger  1        Position line number.  Must be
                                       greater then zero.
               c    UInteger  0        Position column number.
               n    UInteger  34 ('"') Mark name.  Only valid for
                                       GlobalMark and LocalMark
                                       entries.
               f    Binary    N/A      File name.  Required.
               *    any       none     Other keys are allowed for
                                       compatibility reasons, see
                                       |shada-compatibility|.
               -----------------------------------------------------
            */
            match reader.read_exact(&mut tag) {
                Ok(()) => {},
                Err(_) => { panic!("no tag") },
            }
            processed = tag.len();

            // println!("LocalMark total_length={}", total_length);

            let mut field_l = 1;
            let mut field_n = 34; // "
            let mut field_f = vec![0_0u8; 0];

            while processed < total_length {
                reader.read_exact(&mut tag).unwrap();
                processed = processed + tag.len();

                reader.read_exact(&mut key).unwrap();
                processed = processed + key.len();

                match key[0] as char {
                    'l' => {
                        (length, consumed) = read_uint(&mut reader);
                        processed = processed + consumed;
                        field_l = length;
                    },
                    'c' => {
                        (_, consumed) = read_uint(&mut reader);
                        processed = processed + consumed;
                    },
                    'n' => {
                        (length, consumed) = read_uint(&mut reader);
                        processed = processed + consumed;
                        field_n = length;
                    },
                    'f' => {
                        reader.read_exact(&mut tag).unwrap();
                        processed = processed + tag.len();

                        (length, consumed) = read_uint(&mut reader);
                        processed = processed + consumed;

                        let mut filename = vec![0_0u8; length];
                        match reader.read_exact(&mut filename) {
                            Ok(()) => {},
                            Err(_) => { panic!("no filenam") },
                        }
                        processed = processed + length;

                        field_f = filename.clone();
                    },
                    _ => {
                        panic!("unexpected key {}", key[0]);
                    },
                }
            }

            if field_n == 34 && field_f.len() > 0 && field_f[0] == b'/' {
                match str::from_utf8(&field_f) {
                    Ok(v) => println!("{}\t{}", v, field_l),
                    Err(_) => panic!("utf8 convert fail"),
                }
            }
        } else {
            reader.seek_relative(total_length as i64).unwrap();
        }
    }

    // let after = reader.stream_position().unwrap();
    // println!("finally ... {}", after);
}
