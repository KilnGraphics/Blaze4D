package graphics.kiln.blaze4d.api;

import graphics.kiln.blaze4d.Blaze4DNatives;
import jdk.incubator.foreign.MemoryAddress;

public class Blaze4DCore {

    private MemoryAddress instance;

    public Blaze4DCore() {
        this.instance = Blaze4DNatives.b4dInit(MemoryAddress.NULL, true);
    }

    public void destroy() {
        Blaze4DNatives.b4dDestroy(this.instance);
        this.instance = null;
    }
}
