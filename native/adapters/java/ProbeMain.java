package lesscode;

import org.apache.commons.csv.CSVFormat;
import org.apache.commons.csv.CSVPrinter;

import java.io.BufferedWriter;
import java.io.IOException;
import java.io.StringWriter;
import java.util.Arrays;
import java.util.List;

public class ProbeMain {
    public static void main(String[] args) {
        String requestId = args.length > 0 ? args[0] : "probe-req-0";
        long startTs = System.currentTimeMillis();

        // Emit start event
        System.out.println("{\"event\":\"start\",\"protocol_version\":1,\"request_id\":\"" + requestId + "\",\"profile_id\":\"java-junit-v1\",\"timestamp\":" + startTs + "}");

        List<CSVFormat> formats = Arrays.asList(
            CSVFormat.DEFAULT,
            CSVFormat.RFC4180,
            CSVFormat.EXCEL,
            CSVFormat.TDF,
            CSVFormat.POSTGRESQL_CSV
        );

        List<Object[]> testCases = Arrays.asList(
            new Object[]{"simple", "value"},
            new Object[]{"with,comma", "next"},
            new Object[]{"with\"quote", "next"},
            new Object[]{"with\r\nnewline", "next"},
            new Object[]{null, "non-null"},
            new Object[]{42, 3.14159},
            new Object[]{Double.NaN, Double.POSITIVE_INFINITY},
            new Object[]{-0.0, 0.0},
            new Object[]{"", "empty"},
            new Object[]{"multi", "column", "row", 123}
        );

        int seq = 0;
        int passed = 0;

        for (int fIdx = 0; fIdx < formats.size(); fIdx++) {
            CSVFormat fmt = formats.get(fIdx);
            for (int cIdx = 0; cIdx < testCases.size(); cIdx++) {
                Object[] row = testCases.get(cIdx);
                for (int pass = 0; pass < 2; pass++) {
                    seq++;
                    String caseId = "case-f" + fIdx + "-c" + cIdx + "-p" + pass;
                    StringWriter sw = new StringWriter();
                    BufferedWriter bw = new BufferedWriter(sw);
                    try (CSVPrinter printer = new CSVPrinter(bw, fmt)) {
                        printer.printRecord(row);
                        printer.flush();
                        String output = sw.toString();
                        long recCount = printer.getRecordCount();
                        
                        // Output state event
                        String jsonOutput = escapeJson(output);
                        System.out.println("{\"event\":\"state\",\"request_id\":\"" + requestId + "\",\"case_id\":\"" + caseId + "\",\"seq\":" + seq + ",\"record_count\":" + recCount + ",\"output_bytes\":" + output.getBytes().length + ",\"output\":\"" + jsonOutput + "\"}");
                        passed++;
                    } catch (IOException e) {
                        System.out.println("{\"event\":\"case_failed\",\"request_id\":\"" + requestId + "\",\"case_id\":\"" + caseId + "\",\"seq\":" + seq + ",\"error\":\"" + escapeJson(e.getMessage()) + "\"}");
                    }
                }
            }
        }

        long endTs = System.currentTimeMillis();
        System.out.println("{\"event\":\"complete\",\"request_id\":\"" + requestId + "\",\"status\":\"success\",\"total_cases\":" + seq + ",\"passed\":" + passed + ",\"timestamp\":" + endTs + "}");
    }

    private static String escapeJson(String s) {
        if (s == null) return "";
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c == '\\') sb.append("\\\\");
            else if (c == '"') sb.append("\\\"");
            else if (c == '\r') sb.append("\\r");
            else if (c == '\n') sb.append("\\n");
            else if (c == '\t') sb.append("\\t");
            else if (c < 32) sb.append(String.format("\\u%04x", (int) c));
            else sb.append(c);
        }
        return sb.toString();
    }
}
