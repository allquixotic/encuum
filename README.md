![encuum image](https://i.imgur.com/CRtF6IH.jpg)

# Encuum - An Enjin Extractor

Encuum is a tool that extracts the contents of Enjin websites onto your disk using a well-known database format called [SQLite](https://sqlite.org/index.html). It does so by accessing the [Enjin API](https://www.enjin.com/api).

## IMPORTANT NOTE

Enjin's guild websites are shutting down at the end of April 2023 according to the developer post [here](https://www.enjin.com/forums/page/6/m/10826/viewthread/33743439-announcing-retirement-enjin-website-builder). If you need to backup your site, it's probably a better idea to start doing so now, to make sure the technology works. Then, set a deadline past which your community's site should be "read-only" as you migrate to a new web hosting service. 

Your read-only date should probably be some time in March at the latest, in case Enjin decides to shutter early, or things start to fail on the site.

## Dependencies

1. You will need to have a valid account in good standing on an Enjin website.

2. You will need a copy of the [Rust programming language](https://www.rust-lang.org/), which is best installed using [these instructions](https://www.rust-lang.org/tools/install).

## Running Encuum

Once you've installed Rust, download a zip archive of Encuum by clicking [here](https://github.com/allquixotic/encuum/archive/refs/heads/master.zip).

Extract the archive somewhere, then open your platform's terminal app:

 - Windows 10 or earlier: Start `cmd.exe` or `Windows PowerShell` using the Start->Run menu or by right-clicking on the start button.
 - Windows 11: Start `Windows Terminal` from the start menu, or you can follow the above instructions for Windows 10.
 - MacOS: Launch the `Terminal` app.
 - Linux: You probably know what your desktop environment's terminal emulator is, already...

1. In your terminal, change directory into the directory where encuum's source code lives (the **extracted** .zip file). For example, if you extracted it to a directory called `encuum`, use the `cd` command to get there. 

2. Create an .env file in the encuum source directory according to the instructions in the below section, `Configuration`.

3. Type `cargo run --release`. This will take a long time, depending on how fast Enjin's API is working, your computer's speed, etc. but expect _approximately_ 10000 forum posts (not threads, but individual posts) to be extracted per hour. For big and crusty forums with many tens of thousands of posts, it may take the better part of a day to extract. It also depends on how big the posts are. Small posts tend to get extracted quickly, while posts with a large amount of content will be delayed on the Enjin server side.

4. 

## Configuration

Create a plain text file called `.env` in the encuum source directory. Then set variable=value type variables for the following parameter. If you don't set a **Required** parameter, the program won't work at all.

 - `email`: **Required**. The email address of your Enjin account.
 - `password`: **Required**. The password of your Enjin account.
 - `website`: **Required**. The domain or subdomain of your Enjin site. To scrape Enjin's help forum, you would just enter "enjin.com" (no quotes). Do NOT include `https://` or anything else in this parameter besides the domain.
 - `database_file`: **Required**. This is a file name that will be created relative to the current directory (where you run this executable) which will contain your site data in [SQLite](https://sqlite.org/index.html) format.
 - `forum_ids`: **Optional.** A comma-separated list of forum IDs to extract into the database. If this field is blank or omitted, encuum will not extract forums. You can obtain a forum's ID by looking at the URL. For example, [this forum](https://www.enjin.com/forums/page/2/m/10826/viewthread/33743439-announcing-retirement-enjin-website-builder)'s number is `10826`. The number you're looking for is after the `/m/` in the URL.
 - `proxy`: **Optional.** Useful for using an HTTP proxy with the extractor, for example to view the content of the HTTP payloads for debugging purposes.

## Example .env file:

```
email=your-enjin-email@example.com
password=your-enjin-password
website=your-domain-or-subdomain.somewhere.com
database_file=your_site.db
forum_ids=12345678,90123456
```

# How to Use your Data After Extraction

Once the program completes, you have a [SQLite database](https://sqlite.org/index.html) with your forum export in it. Many different programs can parse SQLite databases, and transform the data into various formats. See: 

 - https://github.com/planetopendata/awesome-sqlite for a list of useful SQLite tools
 - https://www.dbvis.com for DBVisualizer (freeware with a paid version with extra features)

# Importing Into a New Site

This is beyond the scope of what encuum can help you with, but you will need to use a program (or write a script/program) to transform the data format of encuum's sqlite database into a format that your new site can use, if you want the encuum-exported data to become forum posts on a new site.

I can provide general tips if you give me specifics about where you're trying to import, but I probably won't have time to write code for you.

## Development Status

### Forums

[x] Extracting the list of forum categories
[x] Extracting the list of subforums
[x] Extracting the list of threads
[x] Extracting each forum post, its content and its metadata
[x] Support to stop an extraction and view the incomplete extracted data using SQLite tools
[ ] Support to resume a stopped forum extraction
[ ] Support to update a forum extraction with just the changes
[ ] Saving images, not just links to the images

### Other Enjin features

[ ] Saving users
[ ] Saving wikis
[ ] Saving applications (to join a site)
[ ] Saving private messages
[ ] Saving News posts (via the Enjin News module)
[ ] Saving Gallery images/media
[ ] Saving Minecraft-specific stuff (unlikely to be done by @allquixotic)
[ ] Saving Shop-specific stuff (unlikely to be done by @allquixotic)

### Code features
[x] Support for wait-and-retry when Enjin API times out or fails
[x] Proxy support
[ ] Refactoring
[ ] Bug fixing

# Known Issues

- None yet; I just released this prototype!