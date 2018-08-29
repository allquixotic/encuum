package vacuum;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.List;

import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.core.JsonGenerationException;
import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.ObjectMapper;

public class ForumFileWriter {

	public void writeForumsToFiles(List<Forum> forums) {
		for(Forum f : forums) {
			try {
				writeForum(f);
			}
			catch(Exception e) {
				System.err.println("Failed to serialize " + f.getTitle());
				throw new RuntimeException(e);
			}
		}
	}
	
	public void writeForum(Forum f) throws JsonGenerationException, JsonMappingException, FileNotFoundException, IOException {
		File file = new File(f.getTitle().replaceAll("[^A-Za-z ]", ""));
		ObjectMapper om = new ObjectMapper();
		om.setSerializationInclusion(Include.NON_NULL);
		om.writeValue(new FileOutputStream(file), f);
	}
}
