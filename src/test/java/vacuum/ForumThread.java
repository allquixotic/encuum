package vacuum;

import java.util.List;

import com.fasterxml.jackson.annotation.JsonBackReference;

public class ForumThread extends BaseEntity {

	protected String threadTitle;
	protected String posterName;
	@JsonBackReference
	protected List<Post> replies;
	public String getThreadTitle() {
		return threadTitle;
	}
	public void setThreadTitle(String threadTitle) {
		this.threadTitle = threadTitle;
	}
	public String getPosterName() {
		return posterName;
	}
	public void setPosterName(String posterName) {
		this.posterName = posterName;
	}
	public List<Post> getReplies() {
		return replies;
	}
	public void setReplies(List<Post> replies) {
		this.replies = replies;
	}
	
}
