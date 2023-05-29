use std::{cell::RefCell, fs::read_to_string, rc::Rc};

const SMALL_DIR_SIZE: usize = 100000;
const FILESYSTEM_SPACE: usize = 70000000;
const REQUIRED_SPACE: usize = 30000000;

#[derive(Debug)]
struct File {
    name: String,
    size: usize,
}

type DirRef = Rc<RefCell<Dir>>;

#[derive(Debug)]
struct Dir {
    name: String,
    files: Vec<File>,
    subdirs: Vec<DirRef>,
    parent: Option<DirRef>,
}

struct DirectoryIterator {
    open: Vec<DirRef>,
}

impl Iterator for DirectoryIterator {
    type Item = DirRef;

    fn next(&mut self) -> Option<Self::Item> {
        let dir_ref = self.open.pop();
        if let Some(ref dir_ref) = dir_ref {
            let subdirs = dir_ref.borrow().subdirs.clone();
            self.open.extend(subdirs.into_iter());
        };
        dir_ref
    }
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} file(s)", self.name, self.files.len())
    }
}

impl Dir {
    fn new(name: &str, parent: Option<DirRef>) -> Self {
        Self {
            name: name.to_owned(),
            parent,
            files: Vec::new(),
            subdirs: Vec::new(),
        }
    }

    fn size(&self) -> usize {
        let file_sizes: usize = self.files.iter().map(|f| f.size).sum();
        let dir_sizes: usize = self
            .subdirs
            .iter()
            .map(|subdir| subdir.borrow().size())
            .sum();
        file_sizes + dir_sizes
    }
}

trait DirRefOps
where
    Self: Sized,
{
    fn get_parent(&self) -> Option<Self>;

    fn get_root(&self) -> Self;

    fn get_dir(&self, name: &str) -> Option<Self>;

    fn add_dir(&self, name: &str);

    fn add_file(&self, name: &str, size: usize);

    fn dirs(&self) -> DirectoryIterator;
}

impl DirRefOps for DirRef {
    fn get_parent(&self) -> Option<Self> {
        self.borrow().parent.clone()
    }

    fn get_root(&self) -> Self {
        let mut cwd = self.clone();
        loop {
            let parent = cwd.borrow().parent.clone();
            match parent {
                None => {
                    return cwd;
                }
                Some(p) => cwd = p,
            };
        }
    }

    fn get_dir(&self, name: &str) -> Option<Self> {
        self.borrow()
            .subdirs
            .iter()
            .find(|subdir| subdir.borrow().name == name)
            .cloned()
    }

    fn add_dir(&self, name: &str) {
        let dir = Dir::new(name, Some(self.clone()));
        self.borrow_mut().subdirs.push(Rc::new(RefCell::new(dir)));
    }

    fn add_file(&self, name: &str, size: usize) {
        self.borrow_mut().files.push(File {
            name: name.to_owned(),
            size,
        });
    }

    fn dirs(&self) -> DirectoryIterator {
        DirectoryIterator {
            open: vec![self.clone()],
        }
    }
}

#[derive(Debug)]
enum DirPath {
    To(String),
    Parent,
    Root,
}

impl From<String> for DirPath {
    fn from(s: String) -> Self {
        match s.as_ref() {
            ".." => Self::Parent,
            "/" => Self::Root,
            _ => Self::To(s),
        }
    }
}

#[derive(Debug)]
enum Command {
    ChangeDir(DirPath),
    ListFiles,
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        let s = s.strip_prefix("$ ").unwrap_or(&s);
        match &s[0..2] {
            "cd" => Command::ChangeDir(s[3..].to_owned().into()),
            "ls" => Command::ListFiles,
            _ => panic!("unexpected command type"),
        }
    }
}

#[derive(Debug)]
enum InputLine {
    FileListing(usize, String),
    DirListing(String),
    CommandInvocation(Command),
}

fn main() {
    let input = read_to_string("./input.txt")
        .unwrap()
        .lines()
        .map(|line| {
            if line.starts_with('$') {
                InputLine::CommandInvocation(line.to_owned().into())
            } else {
                let (a, b) = line.split_once(' ').unwrap();
                if a == "dir" {
                    InputLine::DirListing(b.to_owned())
                } else {
                    InputLine::FileListing(a.parse().unwrap(), b.to_owned())
                }
            }
        })
        .collect::<Vec<_>>();

    // Construct file system
    let root = Rc::new(RefCell::new(Dir::new("/", None)));
    let mut cwd = root.clone();
    for line in input {
        match line {
            // Add a file under the current directory
            InputLine::FileListing(size, name) => cwd.add_file(name.as_ref(), size),

            // Add a directory under the current directory
            InputLine::DirListing(name) => cwd.add_dir(name.as_ref()),

            // Change current directory
            InputLine::CommandInvocation(Command::ChangeDir(dir)) => match dir {
                DirPath::To(to) => cwd = cwd.get_dir(to.as_ref()).unwrap(),
                DirPath::Parent => cwd = cwd.get_parent().unwrap(),
                DirPath::Root => cwd = cwd.get_root(),
            },

            // Listing files (no-op)
            InputLine::CommandInvocation(Command::ListFiles) => { /* do nothing */ }
        }
    }

    // Find small directories
    let total_sum_of_small_dirs: usize = root
        .dirs()
        .filter(|dir_ref| dir_ref.borrow().size() <= SMALL_DIR_SIZE)
        .map(|dir_ref| dir_ref.borrow().size())
        .sum();
    println!("[PT1] Total size is {}", total_sum_of_small_dirs);

    // Compute available space and required cleanup amount
    let used_space = root.borrow().size();
    let unused_space = FILESYSTEM_SPACE - used_space;
    let cleanup_space = REQUIRED_SPACE - unused_space;

    // Find smallest directory larger than the required cleanup amount
    let min_big_enough_size = root
        .dirs()
        .filter(|dir_ref| dir_ref.borrow().size() >= cleanup_space)
        .map(|dir_ref| dir_ref.borrow().size())
        .min()
        .unwrap();
    println!("[PT2] Can cleanup folder w/ size {}", min_big_enough_size);
}
