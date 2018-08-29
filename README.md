![encuum image](https://i.imgur.com/CRtF6IH.jpg)

# What is Encuum?

Enjin + vacuum. Figure it out yourself! Read the code! Get off my lawn!

# How?

 - Clone
 - `mvn verify -Dusername=you@enjin-login.tld -Dpassword=yourpassword -Dbaseurl=https://whatever.enjin.com -Dforum=/somewhere/forum`

 
# System property docs

 - username: Email address to login to Enjin
 - password: Password to login to Enjin with
 - baseurl: URL of your Enjin site (custom domain or not doesn't matter) - NO path after the domain name!
 - forum: the 'path' part of the URL to get to your forum module
 - headless: whether or not to use headless Chrome - set to any value to make headless, or set to true/false as desired. Default false.
 - numBrowsers: number of concurrent instances; must be a positive int. Set to Integer.MAX_VALUE (the actual number) for concurrency of as many as you have forums/subforums, but this will use a crazy amount of RAM/CPU on big sites. Default 1.
 
# Requirements

 - Java 8 or later (tested on 8)
 - Windows, probably
 - Chrome
 - A working Maven install
 - Git