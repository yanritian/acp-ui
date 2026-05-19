/// Process stub for web platform
/// This file provides stub implementations for dart:io classes that are not available on web

import 'dart:convert';

/// Stub Process class for web
class Process {
  final int pid;

  Process._(this.pid);

  static Future<Process> start(String executable, List<String> arguments) async {
    throw UnsupportedError('Process.start is not supported on web platform');
  }

  Future<int> kill() async {
    throw UnsupportedError('Process.kill is not supported on web platform');
  }

  Stream<List<int>> get stdout => const Stream.empty();
  Stream<List<int>> get stderr => const Stream.empty();
}

/// Stub IOSink class for web
class IOSink {
  Future<void> write(List<int> bytes) async {
    throw UnsupportedError('IOSink.write is not supported on web platform');
  }

  Future<void> writeln(String line) async {
    throw UnsupportedError('IOSink.writeln is not supported on web platform');
  }

  Future<void> close() async {
    throw UnsupportedError('IOSink.close is not supported on web platform');
  }
}

/// Stub Stream class for web
class ProcessStream {
  Stream<List<int>> get stream => const Stream.empty();
}