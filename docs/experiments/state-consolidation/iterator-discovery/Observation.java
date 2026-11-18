import java.util.Iterator;
import org.apache.commons.csv.CSVFormat;
import org.apache.commons.csv.CSVParser;
import org.apache.commons.csv.CSVRecord;

public final class Observation {
    private static void run(String id, String... operations) throws Exception {
        try (CSVParser parser = CSVParser.parse("oak\nelm\n", CSVFormat.DEFAULT)) {
            Iterator<CSVRecord> a = parser.iterator();
            Iterator<CSVRecord> b = parser.iterator();
            System.out.printf("%s\tstart\t%s\t%d%n", id, a == b, parser.getRecordNumber());
            for (String operation : operations) {
                Iterator<CSVRecord> iterator = operation.endsWith("A") ? a : b;
                String value;
                try {
                    if (operation.equals("close")) {
                        parser.close();
                        value = "closed";
                    } else if (operation.startsWith("hasNext")) {
                        value = Boolean.toString(iterator.hasNext());
                    } else if (operation.startsWith("next")) {
                        value = iterator.next().get(0);
                    } else {
                        throw new AssertionError("Unexpected operation " + operation);
                    }
                } catch (Exception exception) {
                    value = "throws:" + exception.getClass().getSimpleName();
                }
                System.out.printf("%s\t%s\t%s\t%d%n", id, operation, value, parser.getRecordNumber());
            }
        }
    }

    public static void main(String[] args) throws Exception {
        run("H1", "hasNextA", "hasNextA", "nextB", "nextA", "hasNextB", "nextB");
        run("H2", "nextB", "hasNextA", "hasNextB", "nextB", "nextA");
        run("H3", "hasNextA", "close", "hasNextB", "nextA");
    }
}
