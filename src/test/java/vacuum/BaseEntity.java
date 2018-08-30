package vacuum;

import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.databind.ObjectMapper;

public class BaseEntity {

	private static final ObjectMapper om = new ObjectMapper();
	private static final Lock lock = new ReentrantLock();
	protected String url;
	protected String title;

	static {
		om.setSerializationInclusion(Include.NON_NULL);
	}

	public String getUrl() {
		return url;
	}

	public void setUrl(String url) {
		this.url = url;
	}

	public String getTitle() {
		return title;
	}

	public void setTitle(String title) {
		this.title = title;
	}

	public String toString() {
		try(LockWrapper lw = new LockWrapper(lock)) {
			lock.lock();
			return "========" + this.getClass().getSimpleName() + "========\n"
					+ om.writerWithDefaultPrettyPrinter().writeValueAsString(this) + "\n================";
		} catch (Exception e) {
			e.printStackTrace();
			return e.getMessage();
		}
	}
}
