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

// take directory as argument and return sha of the tree
fn create_tree(dir: &str) -> String {
    // get the list of files in the directory
    let entries = fs::read_dir(dir).unwrap();

    // create a vector to store the entries
    let mut entries_vec = Vec::new();

    // loop through the entries
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = {
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            // Use file_name here
            file_name
        };
        let metadata = fs::metadata(&path).unwrap();

        // check if the entry is a directory
        if metadata.is_dir() {
            // ignore the .git directory
            if file_name == ".git" {
                continue;
            }
            
            // push the entry to the vector
            entries_vec.push((40000, file_name, create_tree(&path.to_str().unwrap())));
        } else {
            // get store (mode, name, sha) in the vector. sha is hash of blob

            // load the file content
            let mut file = fs::File::open(&path).unwrap();

            let mut content = Vec::new();
            file.read_to_end(&mut content).unwrap();

            // update the content with the header
            let mut header = format!("blob {}\x00", content.len());
            // header.push_str(std::str::from_utf8(&content).unwrap());
            header.push_str(String::from_utf8_lossy(&content).to_string().as_str());

            // hash the content
            let mut hasher = Sha1::new();
            hasher.update(header.clone());
            let result = hasher.finalize();
            // hash in hex format
            let hash = format!("{:x}", result);

            // store the entry
            entries_vec.push((100644, file_name, hash));

        }
    }

    // sort the entries
    entries_vec.sort_by(|a, b| a.1.cmp(&b.1));
    
    // create the tree content
    let mut tree_content = Vec::new();
    for entry in entries_vec {
        tree_content.push(format!("{:06o} {}\0{}", entry.0, entry.1, entry.2));
    }
    
    // join the tree content
    let tree_content = tree_content.join("");
    
    // create the tree object
    let tree_content = format!("tree {}\0{}", tree_content.len(), tree_content);
    let mut hasher = Sha1::new();
    hasher.update(tree_content.clone());
    let result = hasher.finalize();
    let hash = format!("{:x}", result);
    
    // create the object file
    let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &hash[..2])).unwrap();
    let mut file = fs::File::create(path).unwrap();
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(tree_content.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    file.write_all(&compressed).unwrap();
    
    // return the hash as string
    hash         
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

        // print the hash
        println!("{}", create_tree("."));
        } else {
        println!("unknown command: {}", args[1])
    }
}
