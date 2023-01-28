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

4. Leave the tool running until the console stops updating with messages indicating progress. Make sure your computer doesn't go to sleep while encuum is running.

## Configuration

Create a plain text file called `.env` in the encuum source directory. Then set variable=value type variables for the following parameter. If you don't set a **Required** parameter, the program won't work at all.

| Config Option   | Required | Default | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
|-----------------|----------|---------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `email`         | Yes      | N/A     | The email address of your Enjin account.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `password`      | Yes      | N/A     | The password of your Enjin account.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `website`       | Yes      | N/A     | The domain or subdomain of your Enjin site. For example, to scrape Enjin's help forum, you would just enter "www.enjin.com" (no quotes). Do NOT include `https://`or anything else in this parameter besides the domain.                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `database_file` | Yes      | N/A     | This is a file name that will be created relative to the current directory (where you run this executable) which will contain your site data in [SQLite](https://sqlite.org/index.html) format.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `forum_ids`     | No       | blank   | A comma-separated list of forum IDs to extract into the database. If this field is blank or omitted, encuum will not extract forums. You can obtain a forum's ID by looking at the URL. For example, [this forum](https://www.enjin.com/forums/page/2/m/10826/viewthread/33743439-announcing-retirement-enjin-website-builder)'s number is `10826`. The number you're looking for is after the `/m/` in the URL.                                                                                                                                                                                                                                                            |
| `proxy`         | No       | blank   | Useful for using an HTTP proxy with the extractor, for example to view the content of the HTTP payloads for debugging purposes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `session_id`    | No       | blank   | Useful for specifying a long-lived Enjin Session ID (which gets printed shortly after the program starts up with this option omitted). If you specify a session_id, know that they are valid for approximately 30 days, and may be revoked by Enjin for any reason, requiring you to get a new one. The easiest way to get a new one is to remove this parameter from your .env and re-run the program. If you're running encuum very often, Enjin might stop issuing you Session IDs, so if you're running it, say, dozens of times per minute, it might be a good idea to cache the session ID this way.                                                                          |
| `subforum_ids`  | No       | blank   | A comma-separated list of subforum IDs to extract into the database. **Any subforum whose ID is not included in this list will _not_ be extracted or navigated.** This is useful if you know that you only care about specific subforums and you have a lot of posts in your forum in other subforums that you don't want to backup. Subforum IDs are the number after `/viewforum/` in the Enjin URL. You have to click on a specific subforum to get its ID. The URL path is usually of the form `/someforums/viewforum/12345/m/67890`, where in this example, `12345` is the subforum_id, and `67890` is the preset_id, also known as forum_id or forum instance ID. |
| `keep_going`    | No       | false   | Specify `true` or  `false`  as the value. `true` means we attempt to keep running the script if Enjin returns invalid data to us. This could mask bugs in the encuum code, so make sure to save the output of the program if you turn this on. `false` means that encuum will exit if it receives 5 errors in a row for the same request. For example, if we ask to retrieve a particular forum thread, and get invalid data, or a timeout, 5 times in a row, the program will fail out and exit with  `keep_going=false`. With `keep_going=true`, it will print out the error, but then just move on to the next thread.            |

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

## Troubleshooting Encuum

The error messaging available within Encuum may appear limited, but it's easy to install an intercepting proxy that will give us excellent debug info on what's going on and why a transaction is failing.

If you encounter something like a "Parse Error" while running Encuum, download an intercepting HTTP proxy, such as [Proxyman](https://proxyman.io/). Install it and launch it. You don't have to make an account.

Then, proxyman will show the listening IP address and port at the top of the screen. Plug that info into your `.env` configuration file. For example, if proxyman says it's listening on `http://127.0.0.1:9091` then you'd write this in your `.env` file:

`proxy=http://127.0.0.1:9091`

Once it's running, follow the directions to [enable TLS (aka HTTPS) support in Proxyman](https://docs.proxyman.io/basic-features/ssl-proxying). You may have to run Encuum to whitelist HTTPS decryption of your website's traffic.

Now, re-run Encuum as directed once more via `cargo run --release`. This will cause your Proxyman window to fill up with requests to your guild website. Keep it running until Encuum fails, then copy the "Raw" contents of the last request and response bodies (I need both request *and* response) into a new [GitHub Gist](https://gist.github.com) which you can link to in a [GitHub issue](https://github.com/allquixotic/encuum/issues/new/choose) in this repo. Lastly, *audit the text* of both the request and response, and remove anything sensitive, such as cookie data, session_id parameters, or passwords. Then post your issue, along with the error message you received from Encuum.

By helping me with these reports, we'll work through the remaining bugs in Encuum.

## Development Status

I plan to continue supporting/working on this until Enjin is shut down at the end of April, 2023, after which a tool like this will be of no use to anyone.

If you have any problems, please [file an issue](https://github.com/allquixotic/encuum/issues).

### Forums

 - [x] Extracting the list of forum categories
 - [x] Extracting the list of subforums
 - [x] Extracting the list of threads
 - [x] Extracting each forum post, its content and its metadata
 - [x] Support to stop an extraction and view the incomplete extracted data using SQLite tools
 - [x] Support to download only a specified set of subforums, not the whole entire forum
 - [ ] Support to resume a stopped forum extraction
 - [ ] Support to update a forum extraction with just the changes
 - [ ] Saving images, not just links to the images

### Other Enjin features

 - [ ] Saving users
 - [ ] Saving wikis
 - [ ] Saving applications (to join a site)
 - [ ] Saving private messages
 - [ ] Saving News posts (via the Enjin News module)
 - [ ] Saving Gallery images/media
 - [ ] Saving Minecraft-specific stuff (unlikely to be done by @allquixotic)
 - [ ] Saving Shop-specific stuff (unlikely to be done by @allquixotic)

### Code features
 - [x] Support for wait-and-retry when Enjin API times out or fails
 - [x] Proxy support
 - [ ] Refactoring
 - [ ] Bug fixing

# Known Issues

- None yet; I just released this prototype!

## Licensing

All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0). Direct dependencies such as Rust, Diesel-rs, Hyper and jsonrpsee are licensed under the MIT or 3-clause BSD license, which allow downstream code to have any license.

The license of downstream dependencies (*not* the code in this repo itself) reads as follows:

```
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```