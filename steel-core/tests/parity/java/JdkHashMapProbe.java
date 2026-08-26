import java.util.HashMap;
import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;

/**
 * Differential oracle for Steel's JavaBlockPosSet.
 *
 * Reads an operation script on stdin (one instruction per line):
 *   A <x> <y> <z>   -> insert
 *   S               -> snapshot current iteration order ("x y z" triples joined by ';')
 * and prints one line per S: the iteration order of java.util.HashSet<BlockPos>
 * exactly as ServerExplosion builds its destruction list (vanilla mc26.2,
 * ServerExplosion.java:121-168 uses `new HashSet<>()` then iterates).
 */
public final class JdkHashMapProbe {
    private static final class Pos implements Comparable<Pos> {
        final int x, y, z;
        Pos(int x, int y, int z) { this.x = x; this.y = y; this.z = z; }
        @Override public int hashCode() { return (y + z * 31) * 31 + x; }
        @Override public boolean equals(Object o) {
            if (!(o instanceof Pos p)) return false;
            return x == p.x && y == p.y && z == p.z;
        }
        @Override public int compareTo(Pos p) {
            if (y != p.y) return Integer.compare(y, p.y);
            if (z != p.z) return Integer.compare(z, p.z);
            return Integer.compare(x, p.x);
        }
    }

    public static void main(String[] args) {
        var set = new java.util.HashSet<Pos>();
        var out = new StringBuilder();
        try (Scanner sc = new Scanner(System.in)) {
            while (sc.hasNext()) {
                switch (sc.next()) {
                    case "A" -> set.add(new Pos(sc.nextInt(), sc.nextInt(), sc.nextInt()));
                    case "S" -> {
                        out.setLength(0);
                        for (Pos p : set) {
                            out.append(p.x).append(',').append(p.y).append(',')
                               .append(p.z).append(';');
                        }
                        System.out.println(out);
                        System.out.flush();
                    }
                    default -> throw new AssertionError("bad op");
                }
            }
        }
    }
}
