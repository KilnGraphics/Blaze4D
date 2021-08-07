package me.hydos.rosella.util;

import net.jpountz.xxhash.XXHashFactory;
import org.jetbrains.annotations.NotNull;

import java.nio.charset.StandardCharsets;

/**
 * Utility class to quickly identify and compare entities while retaining a human readable name.
 *
 * Creating instances is (relatively) slow but comparing existing ones is very fast so it is highly
 * recommended to avoid creating new instances when not necessary. (Also reduces typing mistakes)
 */
public class NamedID implements Comparable<NamedID> {

    public final String name;
    public final long id;

    public NamedID(@NotNull String name) {
        assert(!name.isBlank());

        this.name = name;
        byte[] bytes = name.getBytes(StandardCharsets.UTF_8);
        this.id = XXHashFactory.fastestJavaInstance().hash64().hash(bytes, 0, bytes.length, 0);
    }

    @Override
    public int compareTo(@NotNull NamedID other) {
        long diff = this.id - other.id;
        if(diff == 0) {
            return 0;
        }
        if(diff < 0) {
            return -1;
        } else {
            return 1;
        }
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        NamedID namedID = (NamedID) o;
        return id == namedID.id;
    }

    @Override
    public int hashCode() {
        return (int) (id);
    }
}
