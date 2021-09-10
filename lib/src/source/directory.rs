/* pub enum Order {
    Asc,
    Desc,
}

pub enum DirectoryFileSort {
    ModifiedDate(Order),
}

pub struct DirectoryDataSource {
    path: String,
    file_regex: String,
    order: DirectoryFileSort,
}

impl DirectoryDataSource {
    pub fn new(path: &str, file_regex: &str) -> Self {
        Self {
            path: path.into(),
            file_regex: file_regex.into(),
            order: DirectoryFileSort::ModifiedDate(Order::Asc),
        }
    }
}*/
