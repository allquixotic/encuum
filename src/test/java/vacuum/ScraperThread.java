package vacuum;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.openqa.selenium.By;
import org.openqa.selenium.TimeoutException;
import org.openqa.selenium.WebDriver;
import org.openqa.selenium.WebDriverException;
import org.openqa.selenium.WebElement;
import org.openqa.selenium.support.ui.WebDriverWait;

public class ScraperThread implements Runnable {

	private final Forum f;
	private final Supplier<WebDriver> supplier;
	private WebDriver wd;
	private static final Pattern postContentsRx = Pattern.compile("^\\s*\\[quote=@[0-9]+\\](.*?)\\[/quote\\]\\s*$");
	private static boolean terminatedNormally = true;
	
	public ScraperThread(Supplier<WebDriver> supplier, Forum f) {
		this.supplier = supplier;
		this.f = f;
	}
	
	public void run() {
		try {
			wd = supplier.get();
			RunScriptIT.login(wd);
			f.setThreads(new ArrayList<ForumThread>());
			wd.get(f.getUrl());
			RunScriptIT.waitForPageLoad(wd);
			
			//Get the thread links and set up a basic ForumThread object
			WebElement nextArrow = null;
			do {
				if(nextArrow != null) {
					nextArrow.click();
					RunScriptIT.waitForPageLoad(wd);
				}
				for(WebElement we : wd.findElements(By.xpath("//a[contains(@class,'thread-subject')]"))) {
					ForumThread ft = new ForumThread();
					ft.setReplies(new ArrayList<Post>());
					ft.setUrl(we.getAttribute("href"));
					ft.setThreadTitle(we.getText());
					f.getThreads().add(ft);
				}
			} while((nextArrow = nextArrowExists()) != null);
			
			//Get details of each forum thread
			for(ForumThread ft : f.getThreads()) {
				getForumThread(ft);
			}
		}
		catch(Throwable t) {
			t.printStackTrace();
			terminatedNormally = false;
			RunScriptIT.saveWhatchaGot();
			Runtime.getRuntime().exit(1);
		}
		finally {
			if(wd != null) {
				wd.quit();
			}
		}
	}
	
	public boolean wasTerminatedNormally() {
		return terminatedNormally;
	}
	
	
	private WebElement nextArrowExists() {
		try {
			return new WebDriverWait(wd, 5).until(driver -> driver.findElement(By.xpath("//input[@class='right']")));
		} catch(TimeoutException te) { return null; }
	}
	
	private void getForumThread(ForumThread ft) {
		wd.get(ft.getUrl());
		RunScriptIT.waitForPageLoad(wd);
		List<WebElement> usernames = wd.findElements(By.xpath("//a[contains(@class, 'element_username')]"));
		ft.setPosterName(usernames.get(0).getText());
		
		WebElement nextArrow = null;
		do
		{
			if(nextArrow != null) {
				nextArrow.click();
				RunScriptIT.waitForPageLoad(wd);
			}
			List<String> userNameStrings = new ArrayList<String>();
			for(WebElement we : wd.findElements(By.xpath("//a[contains(@class, 'element_username')]"))) {
				userNameStrings.add(we.getText());
			}
			
			int numQuotesOnPage = wd.findElements(By.xpath("//div[contains(@class,'iconf-quote-right')]")).size();
			//Quote each post, strip out the [quote=@whatever][/quote], fill out a Post per each
			for(int i = 1; i <= numQuotesOnPage; i++) {
				Post post = new Post();
				WebElement textArea = null;
				try {
					textArea = wd.findElement(By.xpath("//textarea[@id='content']"));
					textArea.clear();
				}
				catch(WebDriverException wde) { wde.printStackTrace(); continue; }
				
				WebElement currentQuote = wd.findElements(By.xpath("//div[contains(@class,'iconf-quote-right')]")).get(i-1);
				currentQuote.click();
				RunScriptIT.waitForPageLoad(wd);
				
				textArea = wd.findElement(By.xpath("//textarea[@id='content']"));
				String value = textArea.getAttribute("value");
				textArea.clear();
				Matcher m = postContentsRx.matcher(value);
				if(m.matches()) {
					value = m.group(1);
				}
				
				post.setBbcode(value);
				post.setUrl(wd.getCurrentUrl());
				post.setThread(ft);
				post.setPosterName(userNameStrings.get(i-1));
				post.setPostSequenceNumber(i);
				post.setTitle(null);
				ft.getReplies().add(post);
				wd.get(ft.getUrl());
				RunScriptIT.waitForPageLoad(wd);
			}
		} while((nextArrow = nextArrowExists()) != null);
	}

}
