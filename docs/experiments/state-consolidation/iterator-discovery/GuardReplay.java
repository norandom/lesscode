import java.util.ArrayList;
import java.util.Arrays;
import java.util.Iterator;
import java.util.List;
import java.util.NoSuchElementException;
import org.apache.commons.csv.CSVFormat;
import org.apache.commons.csv.CSVParser;
import org.apache.commons.csv.CSVRecord;

public final class GuardReplay {
    private static void observe(List<String> events) throws Exception {
        String id = events.isEmpty() ? "root" : String.join(",", events);
        try (CSVParser parser = CSVParser.parse("oak\nelm\n", CSVFormat.DEFAULT)) {
            Iterator<CSVRecord> iterator = parser.iterator();
            System.out.printf("%s\tstart\tstart\t%d%n", id, parser.getRecordNumber());
            for (String event : events) {
                String result;
                try {
                    result = switch (event) {
                        case "hasNext" -> Boolean.toString(iterator.hasNext());
                        case "close" -> { parser.close(); yield "closed"; }
                        case "next" -> iterator.next().get(0);
                        default -> throw new IllegalArgumentException(event);
                    };
                } catch (NoSuchElementException ex) {
                    result = "throws:" + ex.getClass().getSimpleName();
                }
                System.out.printf("%s\t%s\t%s\t%d%n", id, event, result, parser.getRecordNumber());
            }
        }
    }

    private static void enumerate(List<String> prefix) throws Exception {
        observe(prefix);
        if (prefix.size() < 4) {
            for (String event : List.of("hasNext", "close", "next")) {
                prefix.add(event);
                enumerate(prefix);
                prefix.remove(prefix.size() - 1);
            }
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length == 1 && args[0].equals("all")) {
            enumerate(new ArrayList<>());
        } else if (args.length >= 1 && args[0].equals("witness")) {
            observe(Arrays.asList(args).subList(1, args.length));
        } else {
            throw new IllegalArgumentException("expected all or witness");
        }
    }
}
