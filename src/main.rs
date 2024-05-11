use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use sha1::{Digest, Sha1};


fn init_repo() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory")
}

fn unzip_content(sha: &str) {
    let path = format!(".git/objects/{}/{}", &sha[..2], &sha[2..]);
    let content = fs::read(path).unwrap();
    let decompressed = flate2::read::ZlibDecoder::new(&content[..]);
    let mut s = String::new();
    std::io::BufReader::new(decompressed).read_to_string(&mut s).unwrap();

    // truncate the details before null value and print the content
    let s = s.splitn(2, '\x00').collect::<Vec<&str>>()[1];
    
    print!("{}", s);
}

fn hash_object(file_name: &str) -> String {
    // compute the SHA hash of a Git object. For example, if the contents of a file are hello world, the blob object file would look like this (after Zlib decompression): blob 11\0hello world
    let mut file = fs::File::open(file_name).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();

    let mut hasher = Sha1::new();
    hasher.update(&content);
    let sha = Sha1::digest(&hasher).to_string();

    let mut compressed = Vec::new();
    compressed.extend_from_slice(b"blob ");
    compressed.extend_from_slice(content.len().to_string().as_bytes());
    compressed.push(0);
    compressed.extend_from_slice(&content);
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&compressed).unwrap();
    let compressed = encoder.finish().unwrap();
    let path = format!(".git/objects/{}/{}", &sha[..2], &sha[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &sha[..2])).unwrap();
    fs::write(&path, compressed).unwrap();
    sha
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        init_repo();
    } else if args[1] == "cat-file" {
        // git cat-file -p <blob_sha>
        unzip_content(&args[3]);
    } else if args[1] == "hash-object"{
        // git hash-object -w <file>
        
        println!("{}", hash_object(&args[3]));

    } 
    else {
        println!("unknown command: {}", args[1])
    }
}
