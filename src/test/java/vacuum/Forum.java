package vacuum;

import java.util.List;

import com.fasterxml.jackson.annotation.JsonBackReference;

public class Forum extends BaseEntity {
	
	private List<ForumThread> threads;

	public List<ForumThread> getThreads() {
		return threads;
	}

	public void setThreads(List<ForumThread> threads) {
		this.threads = threads;
	}
	
}
