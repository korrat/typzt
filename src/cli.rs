use clap::{App, Arg};

/// Generate the clap App by using a builer pattern
pub fn build() -> App<'static>
{
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("xylous <xylous.e@gmail.com>")
        .about("CLI tool to manage a digital Zettelkasten")
        .subcommand(App::new("compl")
            .arg(Arg::new("SHELL")
                .required(true))
            .about("generate completion file for a given shell"))
        .subcommand(App::new("new")
            .about("create a new Zettel and print its inbox status and title")
            .arg(Arg::new("inbox")
                .short('i')
                .long("inbox")
                .takes_value(false)
                .help("create the new Zettel in the inbox"))
            .arg(Arg::new("TITLE")
                .required(true)
                .help("title of Zettel")))
        .subcommand(App::new("update")
            .about("update the metadata of a Zettel")
            .arg(Arg::new("FILENAME")
                .required(true)
                .help("path to Zettel")))
        .subcommand(App::new("query")
            .about("return a list of Zettel whose title matches the text")
            .arg(Arg::new("PATTERN")
                .required(true)
                .help("title of Zettel")))
        .subcommand(App::new("find")
            .about("search Zettels by tag")
            .arg(Arg::new("TAG")
                .required(true)
                .help("tag of Zettel")))
        .subcommand(App::new("links")
            .about("list Zettel that <TITLE> links to")
            .arg(Arg::new("TITLE")
                .required(true)
                .help("title of Zettel")))
        .subcommand(App::new("backlinks")
            .about("list files linking to <TITLE>")
            .arg(Arg::new("TITLE")
                .required(true)
                .help("title of Zettel")))
        .subcommand(App::new("search")
            .about("list titles of Zettel that contain provided text")
            .arg(Arg::new("TEXT")
                .required(true)
                .help("text to be searched")))
        .subcommand(App::new("list-tags")
            .about("list all tags registered in the database"))
        .subcommand(App::new("generate")
            .about("(re)generate the database"))
        .subcommand(App::new("not-created")
            .about("list Zettel linked to, but not yet created"))
        .subcommand(App::new("ls")
            .about("list all existing Zettel"))
        .subcommand(App::new("zettelkasten")
            .about("return the path to the Zettelkasten"))
}
