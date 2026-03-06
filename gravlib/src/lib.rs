use std::fs::File;

const DB_EXT: &'static str = ".gravdb";

pub fn initialize_db(name: String) -> Result<(), std::io::Error> {
    let db = File::create_new(name + DB_EXT)?;
    println!("{:?}", db);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
