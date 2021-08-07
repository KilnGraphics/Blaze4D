package me.hydos.rosella.display;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.vkobjects.VkCommon;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Callable;

/**
 * Used to display to something. In general "something" will be a window though in some cases you may want to use extensions to
 */
public abstract class Display {

    // General Display stuff
    public int width;
    public int height;
    public int fps;
    public boolean doVsync;

    // Scheduling related stuff
    //TODO: this


    public Display(int width, int height) {
        this.width = width;
        this.height = height;
        this.doVsync = false;
    }

    public Display(int width, int height, boolean doVsync) {
        this.width = width;
        this.height = height;
        this.doVsync = doVsync;
    }

    /**
     * This method will handle looping for you, meaning you will not have to call update() every frame manually.
     */
    public abstract void startAutomaticLoop(Rosella rosella, Callable<Boolean> loopCallback);

    /**
     * Exit's the {@link Display}. should be called after {@link Rosella} exit's
     */
    public abstract void exit();

    /**
     * Manually updates the window. Best used when you dont have control over when or where the window will need to be updated.
     */
    public void update() {
        calculateFps();
        //FIXME: scheduling is not implemented!
    }

    /**
     * Calculates the Frames Per Second every time update() is called
     */
    protected abstract void calculateFps();

    /**
     * @return The required extensions needed to run the {@link Display}
     */
    public List<String> getRequiredExtensions() {
        return new ArrayList<>();
    }

    /**
     * Creates a surface which can be rendered to by vulkan
     *
     * @return a pointer to a valid surface
     */
    public abstract long createSurface(VkCommon common);

    /**
     * Called when {@link Rosella} has finished initializing so the display can do what it needs to do
     */
    public void onReady() {
    }

    /**
     * When swapchain recreation happens, the size may be 0, 0. we need to wait for the display's size to not be 0, 0. Not doing so may crash {@link Rosella}
     */
    public abstract void waitForNonZeroSize();
}
