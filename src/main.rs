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

fn hash_object(file_name: &str) {
    
    // load the file content
    let mut file = fs::File::open(file_name).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();

    // update the content with the header
    let mut header = format!("blob {}\x00", content.len());
    header.push_str(std::str::from_utf8(&content).unwrap());

    // hash the content
    let mut hasher = Sha1::new();
    hasher.update(header.clone());
    let result = hasher.finalize();
    // hash in hex format
    let hash = format!("{:x}", result);

    // create the object file
    let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &hash[..2])).unwrap();
    let mut file = fs::File::create(path).unwrap();
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(header.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    file.write_all(&compressed).unwrap();

    // print the hash
    println!("{}", hash);
    
}

fn ls_tree(tree_sha: &str) {
    let path = format!(".git/objects/{}/{}", &tree_sha[..2], &tree_sha[2..]);
    let content = fs::read(path).unwrap();
    let decompressed = flate2::read::ZlibDecoder::new(&content[..]);

    // use Vec::new();
    let mut s = Vec::new();
    std::io::BufReader::new(decompressed).read_to_end(&mut s).unwrap();
    
    // // find the first null value and truncate the header
    // let s = std::str::from_utf8(&content).unwrap();
    // let s = s.splitn(2, '\x00').collect::<Vec<&str>>()[1];
    // let content = s.as_bytes();
    
    // split at first null value
    let content = s.splitn(2, |&x| x == 0).collect::<Vec<&[u8]>>()[1];


    // loop through the content
    let mut i = 0;
    while i < content.len() {
        // extract name where 
        //        <mode> <name>\0<20_byte_sha>
        //     <mode> <name>\0<20_byte_sha>
        //     ...
        //     <mode> <name>\0<20_byte_sha>

        // skip mode and whitespace
        i += 6;

        // extract name by finding null value index
        let name_end = content[i..].iter().position(|&x| x == 0).unwrap();
        let name = String::from_utf8_lossy(&content[i..i+name_end]).to_string();
        i += name_end + 21;

        //remove any trailing whitespace
        let name = name.trim();
    

        // print the name
        println!("{}", name);
    }


//     The format of a tree object file looks like this (after Zlib decompression):
//     tree <size>\0
//     <mode> <name>\0<20_byte_sha>
//     <mode> <name>\0<20_byte_sha>
//     ...
//     <mode> <name>\0<20_byte_sha>
//     For files, the valid values are:
//     100644 (regular file)
//     100755 (executable file)
//     120000 (symbolic link)
//     For directories, the value is 040000
//     The <name> is the name of the file or directory.
//     The <20_byte_sha> is the SHA-1 hash of the object.

}


fn create_tree() {
    let mut tree_content = Vec::new();
    let entries = fs::read_dir(".").unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file_name = file_name.trim();
        let mut hasher = Sha1::new();
        let mut content = Vec::new();
        let mut header = String::new();
        if path.is_file() {
            hash_object(file_name);
        } else if path.is_dir() {
            // recursively create a tree object
            create_tree();
        }
        header.push_str(std::str::from_utf8(&content).unwrap());
        hasher.update(header.clone());
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        let mode = if path.is_file() {
            "100644"
        } else if path.is_dir() {
            "040000"
        } else {
            // handle other file types if needed
            "100644"
        };
        tree_content.push((mode, file_name, hash));

    let mut tree_content_str = String::new();
    for (mode, name, hash) in tree_content {
        tree_content_str.push_str(&format!("{} {} {}\0", mode, name, hash));
    }

    let mut hasher = Sha1::new();
    hasher.update(tree_content_str.clone());
    let result = hasher.finalize();
    let hash = format!("{:x}", result);

    let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &hash[..2])).unwrap();
    let mut file = fs::File::create(path).unwrap();
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(tree_content_str.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    file.write_all(&compressed).unwrap();

    println!("{}", hash);
}


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        // git init
        init_repo();
    } else if args[1] == "cat-file" {
        // git cat-file -p <blob_sha>

        // check if the args[2] is -p
        if args[2] != "-p" {
            println!("unknown option: {}", args[2]);
            return;
        }

        unzip_content(&args[3]);
    } else if args[1] == "hash-object"{
        // git hash-object -w <file>

        // check if the args[2] is -w
        if args[2] != "-w" {
            println!("unknown option: {}", args[2]);
            return;
        }

        hash_object(&args[3]);
    } else if args[1] == "ls-tree" {
        // git ls-tree --name-only <tree_sha>

        // check if the args[2] is --name-only
        if args[2] != "--name-only" {
            println!("unknown option: {}", args[2]);
            return;
        }

        ls_tree(&args[3]);

    } else if args[1] == "write-tree" {
        // git write-tree

        create_tree();
    }else {
        println!("unknown command: {}", args[1])
    }
}
