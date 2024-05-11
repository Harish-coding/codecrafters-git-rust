use std::env;
use std::fs;
use std::io::Read;


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
    println!("{}", &s[..s.len()-1]);
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
    } else {
        println!("unknown command: {}", args[1])
    }
}
