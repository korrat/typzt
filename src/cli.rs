use clap::{Arg, Command};

/// Generate the clap App by using a builer pattern
pub fn build() -> Command<'static>
{
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("xylous <xylous.e@gmail.com>")
        .about("CLI tool to manage a digital Zettelkasten")
        .subcommand(
            Command::new("compl")
                .arg(Arg::new("SHELL").required(true))
                .about("generate completion file for a given shell"),
        )
        .subcommand(
            Command::new("new")
                .about("create a new Zettel and print its inbox status and title")
                .arg(
                    Arg::new("PROJECT")
                        .short('p')
                        .long("project")
                        .takes_value(true)
                        .help("create the new Zettel in a specified project"),
                )
                .arg(Arg::new("TITLE").required(true).help("title of Zettel")),
        )
        .subcommand(
            Command::new("mv")
                .about("move all matches into the given project")
                .arg(
                    Arg::new("PATTERN")
                        .required(true)
                        .help("a pattern/regex for the Zettel titles"),
                )
                .arg(
                    Arg::new("PROJECT")
                        .required(true)
                        .help("the project into which notes are put"),
                ),
        )
        .subcommand(
            Command::new("rename")
                .about("rename a Zettel")
                .arg(Arg::new("TITLE").required(true))
                .arg(Arg::new("NEW_TITLE").required(true)),
        )
        .subcommand(
            Command::new("update")
                .about("update the metadata of a Zettel")
                .arg(Arg::new("FILENAME").required(true).help("path to Zettel")),
        )
        .subcommand(
            Command::new("query")
                .about("return a list of Zettel whose title matches the text")
                .arg(
                    Arg::new("PATTERN")
                        .required(true)
                        .help("pattern for the title of Zettel"),
                ),
        )
        .subcommand(
            Command::new("find")
                .about("search Zettels by tag")
                .arg(Arg::new("TAG").required(true).help("tag of Zettel")),
        )
        .subcommand(
            Command::new("links")
                .about("list Zettel that <TITLE> links to")
                .arg(Arg::new("TITLE").required(true).help("title of Zettel")),
        )
        .subcommand(
            Command::new("backlinks")
                .about("list files linking to <TITLE>")
                .arg(Arg::new("TITLE").required(true).help("title of Zettel")),
        )
        .subcommand(
            Command::new("search")
                .about("list titles of Zettel that contain provided text")
                .arg(Arg::new("TEXT").required(true).help("text to be searched")),
        )
        .subcommand(Command::new("tags").about("list all tags registered in the database"))
        .subcommand(Command::new("projects").about("list all projects within the Zettelkasten"))
        .subcommand(Command::new("generate").about("(re)generate the database"))
        .subcommand(Command::new("ghosts").about("list Zettel mentioned, but not created"))
        .subcommand(Command::new("ls").about("list all existing Zettel"))
        .subcommand(Command::new("zk").about("return the path to the Zettelkasten"))
        .subcommand(Command::new("isolated").about(
            "list all Zettel (in the main zettelkasten) that aren't linked with other notes",
        ))
}
