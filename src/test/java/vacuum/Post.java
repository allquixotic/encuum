package vacuum;

public class Post extends BaseEntity {

	protected String posterName;
	protected ForumThread thread;
	protected String bbcode;
	protected int postSequenceNumber;
	
	
	public int getPostSequenceNumber() {
		return postSequenceNumber;
	}
	public void setPostSequenceNumber(int postSequenceNumber) {
		this.postSequenceNumber = postSequenceNumber;
	}
	public String getPosterName() {
		return posterName;
	}
	public void setPosterName(String posterName) {
		this.posterName = posterName;
	}
	public ForumThread getThread() {
		return thread;
	}
	public void setThread(ForumThread thread) {
		this.thread = thread;
	}
	public String getBbcode() {
		return bbcode;
	}
	public void setBbcode(String bbcode) {
		this.bbcode = bbcode;
	}
}
