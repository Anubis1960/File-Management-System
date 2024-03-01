use std::{cmp, fs};
use std::path::{Path, PathBuf};
extern crate fs_extra;
use fs_extra::dir::{get_size, remove};
use std::fs::File;
use std::io;
use std::io::Read;

/**
Include the following in your Cargo.toml file:
[dependencies]
fs_extra = "1.3.0"
**/

const FNV_PRIME: u64 = 1099511628211;
const FNV_OFFSET_BASIS: u64 = 	14695981039346656037;

#[derive(Debug)]
#[derive(PartialEq)]
enum FileType {
    File,
    Directory,
}

impl Clone for FileType {
    fn clone(&self) -> Self {
        match self {
            FileType::File => FileType::File,
            FileType::Directory => FileType::Directory,
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
struct FileMetadata {
    name: String,
    path: PathBuf,
    size: u64,
    file_type: FileType,
}

impl Clone for FileMetadata {
    fn clone(&self) -> Self {
        FileMetadata {
            name: self.name.clone(),
            path: self.path.clone(),
            size: self.size,
            file_type: self.file_type.clone(),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
struct HashTable {
    buckets: Vec<Vec<FileMetadata>>,
}

impl HashTable {
    fn new(size: usize) -> Self {
        HashTable {
            buckets: vec![Vec::new(); size],
        }
    }

    fn hash(&self, key: &str) -> usize {
        let mut hash: u64 = FNV_OFFSET_BASIS;

        for byte in key.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash as usize
    }

    fn insert(&mut self, file: FileMetadata) {
        let name = format!("{:?}{:?}{}", file.name,file.path,file.size);
        let hash = self.hash(&name);
        let len = self.buckets.len();
        let files = &mut self.buckets[hash % len];
        files.push(file.clone());
    }
}

impl Clone for HashTable {
    fn clone(&self) -> Self {
        HashTable {
            buckets: self.buckets.clone(),
        }
    }
}

fn build_hash_table(path: &Path, mut hash_table: HashTable) -> Option<HashTable>{
    for entry in fs::read_dir(path).ok()? {
        match entry {
            Ok(entry) => {
                let file_type = entry.file_type().unwrap();

                if file_type.is_file() {
                    continue;
                } else if file_type.is_dir() {
                    let dir_size = get_size(&entry.path());
                    match dir_size {
                        Ok(size) => {
                            let file_metadata = FileMetadata {
                                name: entry.file_name().to_string_lossy().into(),
                                path: entry.path(),
                                size,
                                file_type: FileType::Directory,
                            };
                            hash_table.insert(file_metadata);
                            hash_table = build_hash_table(&entry.path(), hash_table)?;
                        }
                        Err(e) => {
                            println!("Error reading directory: {}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error reading directory: {}", e);
                continue;
            }
        }
    }
    Some(hash_table)
}

fn print_hash_table(hash_table: &HashTable) {
    let mut bucket_count = 0;
    for (bucket_index, files) in hash_table.buckets.iter().enumerate() {
        if files.is_empty() {
            continue;
        }
        bucket_count += 1;
        for file in files {
            println!(
                "Bucket {}: {} ({:?} Name: {} - {} bytes - {} kilobytes - {} megabytes)",
                bucket_index,
                file.path.display(),
                file.file_type,
                file.name,
                file.size,
                file.size as f32 / 1024.0,
                file.size as f32 / 1024.0 / 1024.0,
            );
        }
    }
    println!("Total buckets: {}", bucket_count);
    println!("Load factor {}", bucket_count as f32 / hash_table.buckets.len() as f32);
}

#[derive(Debug)]
struct AVLTreeNode {
    file: Option<FileMetadata>,
    left: Option<Box<AVLTreeNode>>,
    right: Option<Box<AVLTreeNode>>,
    parent: Option<Box<AVLTreeNode>>,
    height: i32,
}

impl AVLTreeNode {
    fn new(file: FileMetadata) -> Self {
        AVLTreeNode {
            file: Some(file),
            left: None,
            right: None,
            parent: None,
            height: 1,
        }
    }
}

impl Clone for AVLTreeNode {
    fn clone(&self) -> Self {
        AVLTreeNode {
            file: self.file.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            parent: self.parent.clone(),
            height: self.height,
        }
    }
}

fn build_avl_tree(path: &Path, avlvec: &mut Vec<Option<Box<AVLTreeNode>>>) -> Option<Box<AVLTreeNode>> {
    let mut root = None;

    for entry in fs::read_dir(path).ok()?{
        match entry {
            Ok(entry) => {
                let file_type = entry.file_type().unwrap();
                let metadata = entry.metadata().unwrap();

                if file_type.is_file() {
                    let file_metadata = FileMetadata {
                        name: entry.file_name().to_string_lossy().into(),
                        path: entry.path(),
                        size: metadata.len(),
                        file_type: FileType::File,
                    };
                    root = Some(insert_into_avl_tree(root, file_metadata));
                } else if file_type.is_dir() {
                    build_avl_tree(&entry.path(), avlvec);
                }
                else{
                    let file_metadata = FileMetadata {
                    name: entry.file_name().to_string_lossy().into(),
                    path: entry.path(),
                    size: metadata.len(),
                    file_type: FileType::Directory,
                    };
                    root = Some(insert_into_avl_tree(root, file_metadata));
                }
            }
            Err(e) => {
                println!("Error reading directory: {}", e);
                continue;
            }
        }
    }
    //println!("AVL Tree for directory: {:?}:", path);
    //print_avl_tree(&root, 0);
    avlvec.push(root.clone());
    root
}

fn insert_into_avl_tree(root: Option<Box<AVLTreeNode>>, file: FileMetadata) -> Box<AVLTreeNode> {
    let mut root = root;
    if let Some(node) = root {
        let mut new_node = node.clone();
        if file.name < node.file.as_ref().unwrap().name {
            let left = insert_into_avl_tree(node.left, file);
            new_node.left = Some(left);
        } else {
            let right = insert_into_avl_tree(node.right, file);
            new_node.right = Some(right);
        }
        root = Some(balance_avl_tree(new_node));
    } else {
        root = Some(Box::new(AVLTreeNode::new(file)));
    }
    root.unwrap()
}

fn balance_avl_tree(mut node: Box<AVLTreeNode>) -> Box<AVLTreeNode> {
    update_height(&mut node);
    let balance = get_height(&node.left) - get_height(&node.right);
    if balance > 1 {
        if get_height(&node.left.as_ref().unwrap().left) >= get_height(&node.left.as_ref().unwrap().right) {
            node = rotate_right(node);
        } else {
            node.left = Some(rotate_left(node.left.unwrap()));
            node = rotate_right(node);
        }
    } else if balance < -1 {
        if get_height(&node.right.as_ref().unwrap().right) >= get_height(&node.right.as_ref().unwrap().left) {
            node = rotate_left(node);
        } else {
            node.right = Some(rotate_right(node.right.unwrap()));
            node = rotate_left(node);
        }
    }
    node
}

fn rotate_left(mut node: Box<AVLTreeNode>) -> Box<AVLTreeNode> {
    let mut right = node.right.unwrap();
    node.right = right.left.take();
    update_height(&mut node);
    right.left = Some(node);
    update_height(&mut right);
    right
}

fn rotate_right(mut node: Box<AVLTreeNode>) -> Box<AVLTreeNode> {
    let mut left = node.left.unwrap();
    node.left = left.right.take();
    update_height(&mut node);
    left.right = Some(node);
    update_height(&mut left);
    left
}

fn update_height(node: &mut Box<AVLTreeNode>) {
    node.height = 1 + cmp::max(get_height(&node.left), get_height(&node.right));
}


fn get_height(node: &Option<Box<AVLTreeNode>>) -> i32 {
    match node {
        Some(node) => node.height,
        None => 0,
    }
}

fn print_avl_tree(root: &Option<Box<AVLTreeNode>>, level: usize) {
    if let Some(node) = root {
        if let Some(file) = &node.file {
            print_avl_tree(&node.right, level+5);
            for _ in 0..level+3 {
                print!(" ");
            }
            println!(
                "Path: {:?}; {:?} Name: {} - {} bytes",
                file.path,
                file.file_type,
                file.name,
                file.size,
            );
            print_avl_tree(&node.left, level+5);
        }
    }
}

fn search_avl_tree(root: &Option<Box<AVLTreeNode>>, file_path: PathBuf) -> Option<FileMetadata> {
    if let Some(node) = root {
        if let Some(file) = &node.file {
            return if file.path == file_path {
                Some(file.clone())
            } else {
                return if file.path < file_path {
                    search_avl_tree(&node.right, file_path.clone())
                } else {
                    search_avl_tree(&node.left, file_path.clone())
                }
            }
        }
    }
    None
}

fn search_avl_by_name(root: &Option<Box<AVLTreeNode>>, file_name: String) {
    if let Some(node) = root {
        if let Some(file) = &node.file {
            search_avl_by_name(&node.right, file_name.clone());
            if file.name.to_string() == file_name {
                println!("Path: {:?}; {:?} Name: {} - {} bytes", file.path, file.file_type, file.name, file.size,);
            }
            search_avl_by_name(&node.left, file_name.clone());
        }
    }
}

fn main() {

    let mut path_input = String::new();
    println!("Enter the path of the directory: ");

    io::stdin()
        .read_line(&mut path_input)
        .expect("Failed to read line");

    let path = PathBuf::from(path_input.trim());

    let mut num_buckets = String::new();
    println!("Enter the number of buckets: ");

    io::stdin()
        .read_line(&mut num_buckets)
        .expect("Failed to read line");

    let num_buckets: usize = num_buckets.trim().parse().expect("Please type a number!");

    loop {

        let mut avlvec = Vec::new();
        build_avl_tree(&path, &mut avlvec);
        let hash_table = HashTable::new(num_buckets);
        let hash_table = build_hash_table(&path, hash_table).unwrap();
        let mut choice = String::new();

        println!("Enter the number of the option you want to choose: ");
        println!("1. Search for a file");
        println!("2. Search for a directory");
        println!("3. Delete a file");
        println!("4. Delete a directory");
        println!("5. Create a new file");
        println!("6. Create a new directory");
        println!("7. Read from file");
        println!("8. Write to file");
        println!("9. Display AVL Tree");
        println!("10. Display Hash Table");
        println!("11. Exit");

        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        let choice: usize = choice.trim().parse().expect("Please type a number!");

        if choice == 1 {

            let mut file_name = String::new();
            println!("Enter the name of the file you want to search for: ");

            io::stdin()
                .read_line(&mut file_name)
                .expect("Failed to read line");

            let file_name = file_name.trim();
            let file_name = PathBuf::from(file_name);

            for root in &avlvec {
                search_avl_by_name(root, file_name.clone().into_os_string().into_string().unwrap());
            }

        } else if choice == 2 {

            let mut dir_name = String::new();
            println!("Enter the name of the directory you want to search for: ");

            io::stdin()
                .read_line(&mut dir_name)
                .expect("Failed to read line");

            let dir_name = dir_name.trim();
            let dir_name = PathBuf::from(dir_name);
            let mut found = false;

            for root in hash_table.clone().buckets {
                for file in root {
                    if file.name == dir_name.to_string_lossy() {
                        println!("Directory: {:?} - {} bytes - {} kilobytes - {} megabytes ",
                                 file.path,
                                 file.size,
                                 file.size as f32 / 1024.0,
                                 file.size as f32 / 1024.0 / 1024.0);
                        found = true;
                    }
                }
            }

            if !found {
                println!("Directory not found!");
            }

        } else if choice == 3 {

            let mut file_name = String::new();
            println!("Enter the path of the file you want to delete: ");

            io::stdin()
                .read_line(&mut file_name)
                .expect("Failed to read line");

            let file_name = file_name.trim();
            let file_name = PathBuf::from(file_name);
            let mut found = false;

            for root in &avlvec {
                if search_avl_tree(root, file_name.clone()).is_some() {
                    fs_extra::file::remove(&file_name).expect("Failed to remove file");
                    println!("File removed successfully!");
                    found = true;
                }
            }

            if !found {
                println!("File not found!");
            }

        } else if choice == 4 {

            let mut dir_name = String::new();
            println!("Enter the path of the directory you want to delete: ");

            io::stdin()
                .read_line(&mut dir_name)
                .expect("Failed to read line");

            let dir_name = dir_name.trim();
            let dir_name = PathBuf::from(dir_name);
            let mut found = false;

            for root in hash_table.clone().buckets {
                for file in root {
                    if file.path == dir_name {
                        remove(dir_name.clone()).expect("Failed to remove directory");
                        println!("Directory removed successfully!");
                        found = true;
                    }
                }
            }

            if !found {
                println!("Directory not found!");
            }

        }else if choice == 5 {

            let mut file_name = String::new();
            println!("Enter the path of the file you want to create: ");

            io::stdin()
                .read_line(&mut file_name)
                .expect("Failed to read line");

            let file_name = file_name.trim();
            let file_name = PathBuf::from(file_name);
            File::create(file_name.clone()).expect("Failed to create file");
            println!("File created successfully!");

        }else if choice == 6 {

            let mut dir_name = String::new();
            println!("Enter the path of the directory you want to create: ");

            io::stdin()
                .read_line(&mut dir_name)
                .expect("Failed to read line");

            let dir_name = dir_name.trim();
            let dir_name = PathBuf::from(dir_name);
            fs::create_dir(dir_name.clone()).expect("Failed to create directory");
            println!("Directory created successfully!");

        }else if choice == 7{

            let mut file_name = String::new();
            println!("Enter the path to the file you want to read from: ");

            io::stdin()
                .read_line(&mut file_name)
                .expect("Failed to read line");

            let file_name = file_name.trim();
            let file_name = PathBuf::from(file_name);
            let mut found = false;

            for root in &avlvec {
                if search_avl_tree(root, file_name.clone()).is_some() {
                    found = true;
                    let mut file = File::open(file_name.clone()).expect("Failed to open file");
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).expect("Failed to read file");
                    println!("File contents: {}", contents);
                }
            }

            if !found {
                println!("File not found!");
            }

        }else if choice == 8 {

            let mut file_name = String::new();
            println!("Enter the path to the file you want to write to: ");

            io::stdin()
                .read_line(&mut file_name)
                .expect("Failed to read line");

            let file_name = file_name.trim();
            let file_name = PathBuf::from(file_name);
            let mut found = false;

            for root in &avlvec {
                if search_avl_tree(root, file_name.clone()).is_some() {

                    let mut new_contents = String::new();
                    println!("Enter the new contents of the file: ");

                    io::stdin()
                        .read_line(&mut new_contents)
                        .expect("Failed to read line");

                    fs_extra::file::write_all(file_name.clone(), new_contents.as_str()).expect("Failed to write to file");
                    println!("File written to successfully!");
                    found = true;

                }
            }

            if !found {
                println!("File not found!");
            }

        }else if choice == 9 {
            for root in &avlvec {
                print_avl_tree(root, 0);
                println!();
            }
        } else if choice == 10 {
            print_hash_table(&hash_table.clone());
        } else if choice == 11 {
            break;
        } else {
            println!("Invalid choice!");
        }

        drop(avlvec);
        drop(hash_table);

    }

}