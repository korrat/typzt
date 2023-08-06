use crate::{config::ConfigOptions, str_to_vec, zettel::Zettel};
use rayon::prelude::*;
use rusqlite::{
    named_params, Connection, DatabaseName, Error, Result, Row, Transaction, TransactionBehavior,
};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

impl Zettel
{
    /// Construct a Zettel from an entry in the database metadata
    /// Return an Error if the `row` was invalid
    fn from_db(row: &Row) -> Result<Zettel, rusqlite::Error>
    {
        let title: String = row.get(0)?;
        let project: String = row.get(1)?;
        let links: String = row.get(2)?;
        let tags: String = row.get(3)?;
        let mut z = Zettel::new(&title, &project);
        z.links = str_to_vec(&links);
        z.tags = str_to_vec(&tags);
        Ok(z)
    }
}

pub struct Database
{
    conn: Arc<Mutex<Connection>>,
}

impl Database
{
    /// Create a `Database` interface to an SQLite database
    /// Return an Error if the connection couldn't be made
    pub fn new(uri: &str) -> Result<Self, Error>
    {
        Ok(Database { conn: Arc::new(Mutex::new(Connection::open(uri)?)) })
    }

    /// Create a `Database` interface to a named SQLite database, opened in memory
    /// Return an Error if the connection couldn't be made
    pub fn new_in_memory(filename: &str) -> Result<Self, Error>
    {
        let uri = &format!("file:{}?mode=memory&cache=shared", filename);
        Database::new(uri)
    }

    /// Initialise the current Database with a `zettelkasten` table that holds the properties of
    /// `Zettel`s, if it doesn't exist already
    /// Return an Error if this wasn't possible
    pub fn init(&self) -> Result<(), Error>
    {
        self.conn.lock().unwrap().execute(
                                           "CREATE TABLE IF NOT EXISTS zettelkasten (
                title       TEXT NOT NULL,
                project     TEXT,
                links       TEXT,
                tags        TEXT,
                UNIQUE(title, project)
            )",
                                           [],
        )?;
        Ok(())
    }

    /// Save current Database to `path`
    /// Return an Error if this wasn't possible
    pub fn write_to(&self, path: &str) -> Result<(), Error>
    {
        self.conn
            .lock()
            .unwrap()
            .backup(DatabaseName::Main, path, None)?;
        Ok(())
    }

    /// Save a Zettel's metadata to the database
    pub fn save(&self, zettel: &Zettel) -> Result<(), Error>
    {
        let links = crate::vec_to_str(&zettel.links);
        let tags = crate::vec_to_str(&zettel.tags);
        self.conn.lock().unwrap().execute(
            "INSERT INTO zettelkasten (title, project, links, tags) values (?1, ?2, ?3, ?4)",
            [&zettel.title, &zettel.project, &links, &tags],
        )?;
        Ok(())
    }

    /// Delete a Zettel's metadata from the database
    pub fn delete(&self, zettel: &Zettel) -> Result<(), Error>
    {
        self.conn
            .lock()
            .unwrap()
            .execute("DELETE FROM zettelkasten WHERE title=?1 AND project=?2",
                     [&zettel.title, &zettel.project])?;
        Ok(())
    }

    /// Return all Zettel in the database
    /// Return an Error if the data in a row couldn't be accessed or if the database was
    /// unreachable
    pub fn all(&self) -> Result<Vec<Zettel>, Error>
    {
        let conn_lock = self.conn.lock().unwrap();
        let mut stmt = conn_lock.prepare("SELECT * FROM zettelkasten")?;
        let mut rows = stmt.query([])?;

        let mut results: Vec<Zettel> = Vec::new();
        while let Some(row) = rows.next()? {
            let zettel = Zettel::from_db(row)?;
            results.push(zettel);
        }

        Ok(results)
    }

    /// Search in the database for the Zettels whose `title` property matches `pattern`, and return
    /// them
    /// Return an Error if the databases was unreachable.
    ///
    /// `pattern` uses SQL pattern syntax, e.g. `%` to match zero or more characters.
    pub fn find_by_title(&self, pattern: &str) -> Result<Vec<Zettel>, Error>
    {
        let conn_lock = self.conn.lock().unwrap();
        let mut stmt = conn_lock.prepare("SELECT * FROM zettelkasten WHERE title LIKE :pattern")?;
        let mut rows = stmt.query(named_params! {":pattern": pattern})?;

        let mut results: Vec<Zettel> = Vec::new();
        while let Some(row) = rows.next()? {
            let zettel = Zettel::from_db(row)?;
            results.push(zettel);
        }

        Ok(results)
    }

    /// Return a list of all unique tags found in the database
    ///
    /// Return an Error if the database was unreachable
    pub fn list_tags(&self) -> Result<Vec<String>, Error>
    {
        let conn_lock = self.conn.lock().unwrap();
        let mut stmt = conn_lock.prepare("SELECT tags FROM zettelkasten")?;
        let mut rows = stmt.query([])?;

        let mut results: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            let tags: String = row.get(0)?;
            for tag in str_to_vec(&tags) {
                results.push(tag);
            }
        }
        results.par_sort();
        results.dedup();
        Ok(results)
    }

    /// Return a list of all unique project names found in the database
    ///
    /// Return an Error if the database was unreachable
    pub fn list_projects(&self) -> Result<Vec<String>, Error>
    {
        let conn_lock = self.conn.lock().unwrap();
        let mut stmt = conn_lock.prepare("SELECT project FROM zettelkasten")?;
        let mut rows = stmt.query([])?;

        let mut results: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            let project: String = row.get(0)?;
            if !project.is_empty() {
                results.push(project);
            }
        }
        results.par_sort();
        results.dedup();
        Ok(results)
    }

    /// Search in the database for Zettel that have been linked to, but don't yet exist
    /// Return an Error if the database was unreachable or if the data in a Row couldn't have been
    /// accessed
    pub fn zettel_not_yet_created(&self) -> Result<Vec<String>>
    {
        let conn_lock = self.conn.lock().unwrap();
        let mut stmt = conn_lock.prepare("SELECT links FROM zettelkasten")?;
        let mut rows = stmt.query([])?;

        let mut unique_links: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            let links_str: String = row.get(0)?;
            let links = str_to_vec(&links_str);
            unique_links.extend(links);
        }

        unique_links.par_sort();
        unique_links.dedup();

        Ok(unique_links.into_iter()
                       .filter(|link| {
                           // if the response was empty, then nothing has been found, meaning it doesn't exist
                           // in the database
                           self.find_by_title(link).unwrap().is_empty()
                       })
                       .collect())
    }

    /// Look for Markdown files in the Zettelkasten directory and populate the database with their
    /// metadata
    pub fn generate(&self, cfg: &ConfigOptions) -> Result<(), Error>
    {
        let mut directories = crate::io::list_subdirectories(&cfg.zettelkasten);

        let (tx, rx) = mpsc::sync_channel::<String>(1);
        let conn = self.conn.clone();
        let data_sep: &str = "=?=";

        // Add a separate thread to handle transactioning everything at once
        thread::spawn(move || {
            let conn_lock = conn.lock().unwrap();
            let tsx =
                Transaction::new_unchecked(&conn_lock, TransactionBehavior::Immediate).unwrap();
            let stmt =
                "INSERT INTO zettelkasten (title, project, links, tags) values (?1, ?2, ?3, ?4)";
            loop {
                let data = rx.recv();
                match data {
                    Ok(s) => {
                        let res: Vec<&str> = s.split(data_sep).collect();
                        tsx.execute(stmt, [res[0], res[1], res[2], res[3]]).unwrap();
                    }
                    // If we get a RecvError, then we know we've encountered the end
                    Err(mpsc::RecvError) => {
                        tsx.commit().unwrap();
                        return;
                    }
                }
            }
        });

        directories.push(cfg.zettelkasten.clone());
        directories.par_iter().for_each(|dir| {
                                    let paths: Vec<String> =
                                        // don't add markdown file that starts with a dot (which
                                        // includes the empty title file, the '.md')
                                        crate::io::list_md_files(dir).into_iter()
                                                                    .filter(|f| {
                                                                        !crate::io::basename(f).starts_with('.')
                                                                    })
                                                                    .collect();
                                    paths.par_iter().for_each(|path| {
                                                    let zettel = Zettel::from_file(cfg, path);
                                                    let links = crate::vec_to_str(&zettel.links);
                                                    let tags = crate::vec_to_str(&zettel.tags);
                                                    let data = [zettel.title, zettel.project, links, tags].join(data_sep);
                                                    tx.send(data).unwrap();
                                    });
        });
        // Send RecvError to the thread
        drop(tx);

        Ok(())
    }

    /// Update the metadata for a given Zettel. The specified path *must* exist
    /// Not practical for a bunch of Zettel. Use `generate` instead.
    pub fn update(&self, cfg: &ConfigOptions, zettel: &Zettel) -> Result<(), Error>
    {
        self.delete(zettel)?;
        let z = &Zettel::from_file(cfg, &zettel.filename(cfg));
        self.save(z)?;
        Ok(())
    }

    /// Change the project of the given Zettel within the database
    pub fn change_project(&self, zettel: &Zettel, new_project: &str) -> Result<(), Error>
    {
        self.conn
            .lock()
            .unwrap()
            .execute("UPDATE zettelkasten SET project=?1 WHERE title=?2 AND project=?3",
                     [new_project, &zettel.title, &zettel.project])?;
        Ok(())
    }

    /// Change the title of the given Zettel within the database
    pub fn change_title(&self, zettel: &Zettel, new_title: &str) -> Result<(), Error>
    {
        self.conn
            .lock()
            .unwrap()
            .execute("UPDATE zettelkasten SET title=?1 WHERE title=?2 AND project=?3",
                     [new_title, &zettel.title, &zettel.project])?;
        Ok(())
    }
}
