package me.hydos.rosella;

import me.hydos.rosella.annotations.ExcludeFrequentCI;
import me.hydos.rosella.util.Color;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class TempTest {

    @Test
    void test() {
        Color c = new Color(1.0f, 0.0f, 0.5f, 1.0f);
        assertEquals(c.r(), 255);
        assertEquals(c.g(), 0);
        assertEquals(c.b(), 127);
        assertEquals(c.a(), 255);
    }

    @ExcludeFrequentCI
    @Test
    void test2() {
        Color c = new Color(1.0f, 0.0f, 0.5f, 1.0f);
        assertEquals(c.r(), 255);
        assertEquals(c.g(), 0);
        assertEquals(c.b(), 127);
        assertEquals(c.a(), 255);
    }
}
