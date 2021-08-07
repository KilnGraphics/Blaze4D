package me.hydos.rosella.tests;

import me.hydos.rosella.util.Color;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class TestColor {

    @Test
    void testAsFloats() {
        Color c = new Color(1.0f, 1.0f, 1.0f, 1.0f);
        assertEquals(1.0f, c.rAsFloat(), 1e-6f);
        assertEquals(1.0f, c.gAsFloat(), 1e-6f);
        assertEquals(1.0f, c.bAsFloat(), 1e-6f);

        c = new Color(0.0f, 0.0f, 0.0f, 0.0f);
        assertEquals(0.0f, c.rAsFloat(), 1e-6f);
        assertEquals(0.0f, c.gAsFloat(), 1e-6f);
        assertEquals(0.0f, c.bAsFloat(), 1e-6f);

        c = new Color(0.5f, 0.5f, 0.5f, 0.5f);
        assertEquals(0.5f, c.rAsFloat(), 1e-2f);
        assertEquals(0.5f, c.gAsFloat(), 1e-2f);
        assertEquals(0.5f, c.bAsFloat(), 1e-2f);
    }
}
