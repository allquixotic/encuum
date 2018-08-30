package vacuum;

import java.util.ArrayList;
import java.util.List;

import org.openqa.selenium.By;
import org.openqa.selenium.WebDriver;
import org.openqa.selenium.WebElement;

public class Gatherer {

	public static final String site = System.getProperty("baseurl");
	private static final String forumBase = System.getProperty("forum");
	private static final String baseUrl = site + forumBase;
	
	public List<Forum> getBaseForums(WebDriver wd) {
		List<Forum> retval = new ArrayList<Forum>();
		wd.get(baseUrl);
		RunScriptIT.waitForPageLoad(wd);
		for(WebElement we : wd.findElements(By.xpath("//a[contains(@class,'forum-name') or contains(@class,'subforum-')]"))) {
			Forum f = new Forum();
			f.setUrl(we.getAttribute("href"));
			f.setTitle(we.getText().trim());
			retval.add(f);
		}
		return retval;
	}
	
}
