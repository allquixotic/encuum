package vacuum;

import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.openqa.selenium.By;
import org.openqa.selenium.JavascriptExecutor;
import org.openqa.selenium.WebDriver;
import org.openqa.selenium.chrome.ChromeDriver;
import org.openqa.selenium.chrome.ChromeOptions;
import org.openqa.selenium.support.ui.WebDriverWait;

import com.google.common.base.Strings;

import io.github.bonigarcia.SeleniumExtension;
import io.github.bonigarcia.wdm.WebDriverManager;

@ExtendWith(SeleniumExtension.class)
public class RunScriptIT {
	
	private static final boolean headless = !Strings.isNullOrEmpty(System.getProperty("headless")) ? Boolean.parseBoolean(System.getProperty("headless")) : false;
	private static final int numBrowsers = !Strings.isNullOrEmpty(System.getProperty("numBrowsers")) ? Integer.parseInt(System.getProperty("numBrowsers")) : 1;
	private static ExecutorService es;
	public static List<Forum> forums;
	
	public static WebDriver newChromeDriver() {
		ChromeOptions options = new ChromeOptions();
		List<String> allArguments = new ArrayList<String>();
		if(headless) {
			allArguments.add("headless");
		}
		options.addArguments(allArguments);
		ChromeDriver wd = new ChromeDriver(options);
		wd.manage().timeouts().implicitlyWait(30, TimeUnit.SECONDS);
		return wd;
	}
	
	@BeforeAll
	public static void setUp() {
		final Runtime rt = Runtime.getRuntime();
		try { 
			rt.exec("taskkill /im chrome*");
			Thread.sleep(3000);
			rt.exec("taskkill /f /im chrome*");
			rt.addShutdownHook(new Thread(() -> {
				try {
					rt.exec("taskkill /im chrome*");
					Thread.sleep(1500);
					rt.exec("taskkill /f /im chrome*");
					es.shutdownNow();
				}
				catch(Throwable t) {}
			}));
		}
		catch(Exception e) {}
		Thread t = new Thread( () -> {
			try {
				System.out.println("Type 'q' and <enter> to exit the script.");
				Scanner input = new Scanner(System.in);
				while(input.hasNextLine()) {
					if(input.nextLine().trim().startsWith("q")) {
						rt.exit(0);
					}
				}
			}
			catch(Exception e) {}
		});
		t.start();
		WebDriverManager.chromedriver().setup();
	}
	
    @Test
    public void testStart() throws InterruptedException {
    	Throwable exc = null;
    	try {
	    	WebDriver wd = newChromeDriver();
	    	
	    	login(wd);
	    	
	    	Gatherer g = new Gatherer();
	    	forums = g.getBaseForums(wd);
	    	if(numBrowsers == 1) {
	    		es = Executors.newSingleThreadExecutor();
	    	}
	    	else if(numBrowsers > 1 && numBrowsers < Integer.MAX_VALUE) {
	    		es = Executors.newFixedThreadPool(numBrowsers);
	    	}
	    	else {
	    		es = Executors.newCachedThreadPool();
	    	}
	    	
	    	wd.quit();
	    	
	    	for(Forum f : forums) {
	    		ScraperThread st = new ScraperThread(() -> newChromeDriver(), f);
	    		es.submit(st);
	    	}
	    	es.awaitTermination(20, TimeUnit.DAYS);
	    	es.shutdown();
    	}
    	catch(Throwable t) {
    		t.printStackTrace();
    		exc = t;
    	}
    	finally {
	    	saveWhatchaGot();
    		if(exc != null) Assertions.fail(exc);
    	}
    }
    
    public static void saveWhatchaGot() {
    	ForumFileWriter ffw = new ForumFileWriter();
    	ffw.writeForumsToFiles(forums);
    }
    
    public static void login(WebDriver wd) {
    	wd.get(Gatherer.site + "/login");
    	waitForPageLoad(wd);
    	wd.findElement(By.name("username")).sendKeys(System.getProperty("username"));
    	wd.findElement(By.xpath("//input[@type='password']")).sendKeys(System.getProperty("password"));
    	wd.findElement(By.xpath("//input[@type='submit' and @value='Login']")).click();
    	waitForPageLoad(wd);
    }
    
    public static final void waitForPageLoad(WebDriver wd) {
    	new WebDriverWait(wd, 30).until(driver -> ((JavascriptExecutor) driver).executeScript("return document.readyState").equals("complete"));
    	try { Thread.sleep(1500); } catch(InterruptedException ie) { throw new RuntimeException(ie); }
    }

}