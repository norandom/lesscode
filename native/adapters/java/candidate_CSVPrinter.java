/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *   https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.apache.commons.csv;

import static org.apache.commons.csv.Constants.CR;
import static org.apache.commons.csv.Constants.LF;
import static org.apache.commons.csv.Constants.SP;

import java.io.Closeable;
import java.io.Flushable;
import java.io.IOException;
import java.io.InputStream;
import java.io.Reader;
import java.sql.Blob;
import java.sql.Clob;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Arrays;
import java.util.Objects;
import java.util.concurrent.locks.ReentrantLock;
import java.util.stream.Stream;
import org.apache.commons.io.function.IOStream;

public final class CSVPrinter implements Flushable, Closeable {
    private final Appendable appendable;
    private final CSVFormat format;
    private boolean newRecord = true;
    private long recordCount;
    private final ReentrantLock lock = new ReentrantLock();

    public CSVPrinter(final Appendable appendable, final CSVFormat format) throws IOException {
        this.appendable = Objects.requireNonNull(appendable, "appendable");
        this.format = Objects.requireNonNull(format, "format").copy();
        if (this.format.getHeaderComments() != null) {
            for (final String line : this.format.getHeaderComments()) printComment(line);
        }
        if (this.format.getHeader() != null && !this.format.getSkipHeaderRecord()) {
            printRecord((Object[]) this.format.getHeader());
        }
    }

    @Override
    public void close() throws IOException { close(false); }

    public void close(final boolean flush) throws IOException {
        if (flush || format.getAutoFlush()) flush();
        if (appendable instanceof Closeable) ((Closeable) appendable).close();
    }

    private void endOfRecord() throws IOException { println(); recordCount++; }

    @Override
    public void flush() throws IOException {
        if (appendable instanceof Flushable) ((Flushable) appendable).flush();
    }

    public Appendable getOut() { return appendable; }
    public long getRecordCount() { return recordCount; }

    public void print(final Object value) throws IOException {
        lock.lock();
        try { printRaw(value); } finally { lock.unlock(); }
    }

    private void printRaw(final Object value) throws IOException {
        format.print(value, appendable, newRecord);
        newRecord = false;
    }

    public void println() throws IOException {
        lock.lock();
        try { format.println(appendable); newRecord = true; } finally { lock.unlock(); }
    }

    public void printComment(final String comment) throws IOException {
        lock.lock();
        try {
            if (comment == null || !format.isCommentMarkerSet()) return;
            if (!newRecord) println();
            appendable.append(format.getCommentMarker().charValue()).append(SP);
            for (int i = 0; i < comment.length(); i++) {
                final char c = comment.charAt(i);
                if (c == CR || c == LF) {
                    if (c == CR && i + 1 < comment.length() && comment.charAt(i + 1) == LF) i++;
                    printCommentSeparator();
                    appendable.append(format.getCommentMarker().charValue()).append(SP);
                } else {
                    appendable.append(c);
                }
            }
            printCommentSeparator();
        } finally { lock.unlock(); }
    }

    private void printCommentSeparator() throws IOException {
        if (format.getRecordSeparator() != null) appendable.append(format.getRecordSeparator());
        newRecord = true;
    }

    public void printRecord(final Iterable<?> values) throws IOException {
        lock.lock();
        try { IOStream.of(values).forEachOrdered(this::print); endOfRecord(); } finally { lock.unlock(); }
    }

    public void printRecord(final Object... values) throws IOException { printRecord(Arrays.asList(values)); }

    public void printRecord(final Stream<?> stream) throws IOException {
        lock.lock();
        try {
            IOStream.adapt(stream).forEachOrdered(stream.isParallel() ? this::printRaw : this::print);
            endOfRecord();
        } finally { lock.unlock(); }
    }

    private void printRecordObject(final Object val) throws IOException {
        if (val instanceof Object[]) printRecord((Object[]) val);
        else if (val instanceof Iterable) printRecord((Iterable<?>) val);
        else printRecord(val);
    }

    public void printRecords(final Iterable<?> values) throws IOException { printRecords(IOStream.of(values)); }
    public void printRecords(final Object... values) throws IOException { printRecords(Arrays.asList(values)); }
    public void printRecords(final Stream<?> values) throws IOException { printRecords(IOStream.adapt(values)); }
    private void printRecords(final IOStream<?> stream) throws IOException { format.limit(stream).forEachOrdered(this::printRecordObject); }

    public void printHeaders(final ResultSet rs) throws IOException, SQLException {
        lock.lock();
        try {
            try (IOStream<String> s = IOStream.of(format.builder().setHeader(rs).get().getHeader())) { s.forEachOrdered(this::print); }
            println();
        } finally { lock.unlock(); }
    }

    public void printRecords(final ResultSet rs) throws SQLException, IOException {
        final int cols = rs.getMetaData().getColumnCount();
        long rowCount = 0;
        while (format.useRow(rowCount + 1) && rs.next()) {
            lock.lock();
            try {
                for (int i = 1; i <= cols; i++) {
                    final Object obj = rs.getObject(i);
                    if (obj instanceof Clob) {
                        try (Reader r = ((Clob) obj).getCharacterStream()) { print(r); }
                    } else if (obj instanceof Blob) {
                        try (InputStream in = ((Blob) obj).getBinaryStream()) { print(in); }
                    } else {
                        print(obj);
                    }
                }
                endOfRecord();
            } finally { lock.unlock(); }
            rowCount++;
        }
    }

    public void printRecords(final ResultSet rs, final boolean printHeader) throws SQLException, IOException {
        if (printHeader) printHeaders(rs);
        printRecords(rs);
    }
}
