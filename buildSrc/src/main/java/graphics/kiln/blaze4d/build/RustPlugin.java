package graphics.kiln.blaze4d.build;

import org.gradle.api.Action;
import org.gradle.api.Plugin;
import org.gradle.api.Project;
import org.gradle.api.Task;

public class RustPlugin implements Plugin<Project> {
    @Override
    public void apply(Project project) {
        project.task("buildLibrary").doFirst(new Action<Task>() {
            @Override
            public void execute(Task task) {
                System.out.println("Build has been called!");
            }
        });
    }
}