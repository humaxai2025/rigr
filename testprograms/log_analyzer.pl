#!/usr/bin/perl
#
# Comprehensive Log Analysis System in Perl
#
# This program demonstrates various Perl programming concepts including:
# - Regular expressions and pattern matching
# - File I/O and data processing
# - Hash tables and complex data structures
# - Object-oriented programming with packages/modules
# - Text processing and string manipulation
# - Statistical analysis and reporting
# - Command-line argument processing
# - Error handling and exception management
# - Date/time processing
# - Network log analysis
# - Performance optimization techniques
# - Advanced Perl idioms and best practices

use strict;
use warnings;
use feature qw(say switch);
use Getopt::Long;
use Time::Local;
use POSIX qw(strftime);
use File::Basename;
use Data::Dumper;
use List::Util qw(sum max min reduce);
use IO::File;

# Configuration and constants
our $VERSION = '1.0.0';
our $PROGRAM_NAME = 'Comprehensive Log Analysis System';
our $DEBUG = 0;

# Default configuration
my %config = (
    input_file       => '',
    output_file      => '',
    log_format       => 'combined',
    time_window      => 3600,  # 1 hour
    top_entries      => 10,
    min_frequency    => 1,
    generate_charts  => 0,
    verbose          => 0,
    help             => 0,
);

# Global data structures
my %ip_stats = ();
my %url_stats = ();
my %status_codes = ();
my %user_agents = ();
my %referrers = ();
my %hourly_traffic = ();
my %daily_traffic = ();
my %error_patterns = ();
my @log_entries = ();
my %security_events = ();

# Performance counters
my $total_lines_processed = 0;
my $parse_errors = 0;
my $processing_start_time = time();

#==============================================================================
# Object-Oriented Log Entry Class
#==============================================================================

package LogEntry;

sub new {
    my ($class, %args) = @_;
    my $self = {
        ip_address    => $args{ip_address} || '',
        timestamp     => $args{timestamp} || 0,
        method        => $args{method} || '',
        url           => $args{url} || '',
        protocol      => $args{protocol} || '',
        status_code   => $args{status_code} || 0,
        response_size => $args{response_size} || 0,
        referrer      => $args{referrer} || '',
        user_agent    => $args{user_agent} || '',
        processing_time => $args{processing_time} || 0,
    };
    bless $self, $class;
    return $self;
}

sub is_error {
    my $self = shift;
    return $self->{status_code} >= 400;
}

sub is_success {
    my $self = shift;
    return $self->{status_code} >= 200 && $self->{status_code} < 300;
}

sub is_redirect {
    my $self = shift;
    return $self->{status_code} >= 300 && $self->{status_code} < 400;
}

sub get_hour {
    my $self = shift;
    return strftime('%H', localtime($self->{timestamp}));
}

sub get_date {
    my $self = shift;
    return strftime('%Y-%m-%d', localtime($self->{timestamp}));
}

sub is_suspicious {
    my $self = shift;
    
    # Define suspicious patterns
    my @suspicious_patterns = (
        qr{/\.\.},                    # Directory traversal
        qr{<script},                  # XSS attempts
        qr{union.*select}i,           # SQL injection
        qr{/proc/},                   # System file access
        qr{/etc/passwd},              # Password file access
        qr{cmd\.exe},                 # Command execution
        qr{/admin},                   # Admin interface access
        qr{\.php\?}.*({eval|base64_decode}),  # PHP code injection
    );
    
    for my $pattern (@suspicious_patterns) {
        return 1 if $self->{url} =~ /$pattern/;
        return 1 if $self->{user_agent} =~ /$pattern/;
    }
    
    # Check for brute force patterns
    return 1 if $self->{url} =~ qr{/(login|admin|wp-admin)} && $self->is_error();
    
    # Check for unusual response sizes
    return 1 if $self->{response_size} > 10_000_000;  # 10MB
    
    return 0;
}

sub to_string {
    my $self = shift;
    return sprintf(
        "%s [%s] \"%s %s %s\" %d %d \"%s\" \"%s\"",
        $self->{ip_address},
        strftime('%d/%b/%Y:%H:%M:%S %z', localtime($self->{timestamp})),
        $self->{method},
        $self->{url},
        $self->{protocol},
        $self->{status_code},
        $self->{response_size},
        $self->{referrer},
        $self->{user_agent}
    );
}

package main;

#==============================================================================
# Log Parser Module
#==============================================================================

package LogParser;

# Apache Combined Log Format Parser
sub parse_combined_format {
    my ($line) = @_;
    
    # Combined log format regex
    my $regex = qr{
        ^
        (\S+)                          # IP address
        \s+ \S+ \s+ \S+ \s+           # remote logname and remote user (ignored)
        \[([^\]]+)\]                   # timestamp
        \s+
        "(\S+) \s+ ([^"]*) \s+ ([^"]*)" # method, URL, protocol
        \s+
        (\S+)                          # status code
        \s+
        (\S+)                          # response size
        \s+
        "([^"]*)"                      # referrer
        \s+
        "([^"]*)"                      # user agent
        (.*)                           # optional additional fields
        $
    }x;
    
    if ($line =~ /$regex/) {
        my ($ip, $timestamp_str, $method, $url, $protocol, 
            $status, $size, $referrer, $user_agent, $extra) = 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);
        
        # Parse timestamp
        my $timestamp = parse_timestamp($timestamp_str);
        
        # Clean up fields
        $size = ($size eq '-') ? 0 : $size;
        $referrer = ($referrer eq '-') ? '' : $referrer;
        $user_agent = ($user_agent eq '-') ? '' : $user_agent;
        
        return LogEntry->new(
            ip_address    => $ip,
            timestamp     => $timestamp,
            method        => $method,
            url           => $url,
            protocol      => $protocol,
            status_code   => int($status),
            response_size => int($size),
            referrer      => $referrer,
            user_agent    => $user_agent,
        );
    }
    
    return undef;
}

# Common Log Format Parser
sub parse_common_format {
    my ($line) = @_;
    
    my $regex = qr{
        ^
        (\S+)                          # IP address
        \s+ \S+ \s+ \S+ \s+           # remote logname and remote user
        \[([^\]]+)\]                   # timestamp
        \s+
        "(\S+) \s+ ([^"]*) \s+ ([^"]*)" # method, URL, protocol
        \s+
        (\S+)                          # status code
        \s+
        (\S+)                          # response size
        (.*)                           # optional additional fields
        $
    }x;
    
    if ($line =~ /$regex/) {
        my ($ip, $timestamp_str, $method, $url, $protocol, $status, $size, $extra) = 
            ($1, $2, $3, $4, $5, $6, $7, $8);
        
        return LogEntry->new(
            ip_address    => $ip,
            timestamp     => parse_timestamp($timestamp_str),
            method        => $method,
            url           => $url,
            protocol      => $protocol,
            status_code   => int($status),
            response_size => ($size eq '-') ? 0 : int($size),
        );
    }
    
    return undef;
}

# Parse Apache timestamp format
sub parse_timestamp {
    my ($timestamp_str) = @_;
    
    # Format: 10/Oct/2000:13:55:36 -0700
    if ($timestamp_str =~ /^(\d{2})\/(\w{3})\/(\d{4}):(\d{2}):(\d{2}):(\d{2}) (.+)$/) {
        my ($day, $month_str, $year, $hour, $minute, $second, $tz) = 
            ($1, $2, $3, $4, $5, $6, $7);
        
        my %months = (
            Jan => 0, Feb => 1, Mar => 2, Apr => 3, May => 4,  Jun => 5,
            Jul => 6, Aug => 7, Sep => 8, Oct => 9, Nov => 10, Dec => 11
        );
        
        my $month = $months{$month_str};
        return timelocal($second, $minute, $hour, $day, $month, $year);
    }
    
    return time();  # fallback to current time
}

package main;

#==============================================================================
# Statistical Analysis Functions
#==============================================================================

sub calculate_statistics {
    my ($data_ref) = @_;
    my @data = @$data_ref;
    
    return {} unless @data;
    
    @data = sort { $a <=> $b } @data;
    my $count = @data;
    my $sum = sum(@data);
    my $mean = $sum / $count;
    
    # Calculate median
    my $median;
    if ($count % 2 == 0) {
        $median = ($data[$count/2 - 1] + $data[$count/2]) / 2;
    } else {
        $median = $data[int($count/2)];
    }
    
    # Calculate standard deviation
    my $variance = sum(map { ($_ - $mean) ** 2 } @data) / $count;
    my $std_dev = sqrt($variance);
    
    # Calculate percentiles
    my $p95 = $data[int($count * 0.95)];
    my $p99 = $data[int($count * 0.99)];
    
    return {
        count    => $count,
        sum      => $sum,
        mean     => $mean,
        median   => $median,
        min      => $data[0],
        max      => $data[-1],
        std_dev  => $std_dev,
        p95      => $p95,
        p99      => $p99,
    };
}

sub analyze_ip_patterns {
    my @suspicious_ips;
    
    for my $ip (keys %ip_stats) {
        my $stats = $ip_stats{$ip};
        
        # Flag IPs with suspicious patterns
        if ($stats->{request_count} > 1000 ||           # High request volume
            $stats->{error_rate} > 0.5 ||               # High error rate
            $stats->{suspicious_requests} > 10) {       # Many suspicious requests
            
            push @suspicious_ips, {
                ip => $ip,
                %$stats
            };
        }
    }
    
    return \@suspicious_ips;
}

sub detect_attack_patterns {
    my %attacks;
    
    # SQL Injection detection
    $attacks{sql_injection} = grep {
        $_->{url} =~ /union.*select|drop.*table|insert.*into/i
    } @log_entries;
    
    # XSS detection
    $attacks{xss} = grep {
        $_->{url} =~ /<script|javascript:|onload=|onerror=/i
    } @log_entries;
    
    # Directory traversal
    $attacks{directory_traversal} = grep {
        $_->{url} =~ /\.\.\/|\.\.\\|\.\.[\/\\]/
    } @log_entries;
    
    # Brute force detection
    my %login_attempts;
    for my $entry (@log_entries) {
        if ($entry->{url} =~ m{/(login|admin|wp-login)} && $entry->is_error()) {
            $login_attempts{$entry->{ip_address}}++;
        }
    }
    $attacks{brute_force} = scalar grep { $login_attempts{$_} > 20 } keys %login_attempts;
    
    return \%attacks;
}

#==============================================================================
# Report Generation Functions
#==============================================================================

sub generate_comprehensive_report {
    my ($output_fh) = @_;
    
    print $fh "\n" . "="x80 . "\n";
    print $fh "COMPREHENSIVE LOG ANALYSIS REPORT\n";
    print $fh "Generated: " . strftime('%Y-%m-%d %H:%M:%S', localtime()) . "\n";
    print $fh "="x80 . "\n\n";
    
    # Overview statistics
    generate_overview_report($output_fh);
    
    # Traffic analysis
    generate_traffic_analysis($output_fh);
    
    # Error analysis
    generate_error_analysis($output_fh);
    
    # Security analysis
    generate_security_analysis($output_fh);
    
    # Performance analysis
    generate_performance_analysis($output_fh);
    
    # Top lists
    generate_top_lists($output_fh);
}

sub generate_overview_report {
    my ($fh) = @_;
    
    print $fh "OVERVIEW STATISTICS\n";
    print $fh "-"x50 . "\n";
    print $fh sprintf("Total log entries processed: %d\n", $total_lines_processed);
    print $fh sprintf("Parse errors: %d (%.2f%%)\n", 
        $parse_errors, 
        $total_lines_processed ? ($parse_errors / $total_lines_processed) * 100 : 0);
    
    my $processing_time = time() - $processing_start_time;
    print $fh sprintf("Processing time: %d seconds\n", $processing_time);
    print $fh sprintf("Processing rate: %.2f entries/second\n", 
        $processing_time ? $total_lines_processed / $processing_time : 0);
    
    print $fh "\nUnique visitors: " . (keys %ip_stats) . "\n";
    print $fh "Unique URLs accessed: " . (keys %url_stats) . "\n";
    print $fh "Unique user agents: " . (keys %user_agents) . "\n";
    print $fh "\n";
}

sub generate_traffic_analysis {
    my ($fh) = @_;
    
    print $fh "TRAFFIC ANALYSIS\n";
    print $fh "-"x50 . "\n";
    
    # Hourly traffic distribution
    print $fh "Hourly Traffic Distribution:\n";
    for my $hour (sort keys %hourly_traffic) {
        my $count = $hourly_traffic{$hour};
        my $bar = '#' x int($count / 100);  # Scale bar graph
        print $fh sprintf("%02d:00 [%6d] %s\n", $hour, $count, $bar);
    }
    
    # Daily traffic (if data spans multiple days)
    if (keys %daily_traffic > 1) {
        print $fh "\nDaily Traffic Distribution:\n";
        for my $date (sort keys %daily_traffic) {
            print $fh sprintf("%s: %d requests\n", $date, $daily_traffic{$date});
        }
    }
    
    print $fh "\n";
}

sub generate_error_analysis {
    my ($fh) = @_;
    
    print $fh "ERROR ANALYSIS\n";
    print $fh "-"x50 . "\n";
    
    # Status code distribution
    print $fh "Status Code Distribution:\n";
    for my $code (sort { $a <=> $b } keys %status_codes) {
        my $count = $status_codes{$code};
        my $percentage = ($count / $total_lines_processed) * 100;
        print $fh sprintf("%3d: %6d (%5.2f%%)\n", $code, $count, $percentage);
    }
    
    # Error patterns
    print $fh "\nCommon Error Patterns:\n";
    my %error_urls;
    for my $entry (@log_entries) {
        if ($entry->is_error()) {
            $error_urls{$entry->{url}}++;
        }
    }
    
    my @top_error_urls = sort { $error_urls{$b} <=> $error_urls{$a} } 
                         keys %error_urls;
    
    for my $i (0 .. min(9, $#top_error_urls)) {
        my $url = $top_error_urls[$i];
        print $fh sprintf("%3d. [%4d] %s\n", 
            $i + 1, $error_urls{$url}, $url);
    }
    
    print $fh "\n";
}

sub generate_security_analysis {
    my ($fh) = @_;
    
    print $fh "SECURITY ANALYSIS\n";
    print $fh "-"x50 . "\n";
    
    # Attack pattern detection
    my $attacks = detect_attack_patterns();
    
    print $fh "Potential Attack Attempts:\n";
    for my $attack_type (keys %$attacks) {
        my $count = $attacks->{$attack_type};
        print $fh sprintf("%-20s: %d\n", ucfirst($attack_type), $count);
    }
    
    # Suspicious IP analysis
    my $suspicious_ips = analyze_ip_patterns();
    
    if (@$suspicious_ips) {
        print $fh "\nSuspicious IP Addresses:\n";
        print $fh sprintf("%-15s %8s %8s %8s %8s\n", 
            'IP Address', 'Requests', 'Errors', 'Error%', 'Suspicious');
        print $fh "-"x65 . "\n";
        
        for my $ip_data (sort { $b->{request_count} <=> $a->{request_count} } 
                         @$suspicious_ips) {
            print $fh sprintf("%-15s %8d %8d %7.1f%% %8d\n",
                $ip_data->{ip},
                $ip_data->{request_count},
                $ip_data->{error_count},
                $ip_data->{error_rate} * 100,
                $ip_data->{suspicious_requests}
            );
        }
    }
    
    print $fh "\n";
}

sub generate_performance_analysis {
    my ($fh) = @_;
    
    print $fh "PERFORMANCE ANALYSIS\n";
    print $fh "-"x50 . "\n";
    
    # Response size statistics
    my @response_sizes = map { $_->{response_size} } @log_entries;
    my $size_stats = calculate_statistics(\@response_sizes);
    
    print $fh "Response Size Statistics:\n";
    print $fh sprintf("  Total bytes served: %s\n", format_bytes($size_stats->{sum}));
    print $fh sprintf("  Average response size: %s\n", format_bytes($size_stats->{mean}));
    print $fh sprintf("  Median response size: %s\n", format_bytes($size_stats->{median}));
    print $fh sprintf("  Largest response: %s\n", format_bytes($size_stats->{max}));
    print $fh sprintf("  95th percentile: %s\n", format_bytes($size_stats->{p95}));
    
    # Bandwidth utilization
    my $total_bandwidth = sum(map { $_->{response_size} } @log_entries);
    my $time_span = 3600;  # Assume 1 hour for demo
    my $avg_bandwidth = $total_bandwidth / $time_span;
    
    print $fh sprintf("\nBandwidth Utilization:\n");
    print $fh sprintf("  Average: %s/sec\n", format_bytes($avg_bandwidth));
    
    print $fh "\n";
}

sub generate_top_lists {
    my ($fh) = @_;
    
    print $fh "TOP LISTS\n";
    print $fh "-"x50 . "\n";
    
    # Top IP addresses by request count
    print $fh "Top IP Addresses by Request Count:\n";
    my @top_ips = sort { $ip_stats{$b}->{request_count} <=> $ip_stats{$a}->{request_count} } 
                  keys %ip_stats;
    
    for my $i (0 .. min($config{top_entries} - 1, $#top_ips)) {
        my $ip = $top_ips[$i];
        my $count = $ip_stats{$ip}->{request_count};
        print $fh sprintf("%3d. %-15s: %d requests\n", $i + 1, $ip, $count);
    }
    
    # Top URLs by request count
    print $fh "\nTop URLs by Request Count:\n";
    my @top_urls = sort { $url_stats{$b} <=> $url_stats{$a} } keys %url_stats;
    
    for my $i (0 .. min($config{top_entries} - 1, $#top_urls)) {
        my $url = $top_urls[$i];
        my $count = $url_stats{$url};
        my $display_url = length($url) > 60 ? substr($url, 0, 57) . '...' : $url;
        print $fh sprintf("%3d. [%4d] %s\n", $i + 1, $count, $display_url);
    }
    
    # Top User Agents
    print $fh "\nTop User Agents:\n";
    my @top_agents = sort { $user_agents{$b} <=> $user_agents{$a} } keys %user_agents;
    
    for my $i (0 .. min($config{top_entries} - 1, $#top_agents)) {
        my $agent = $top_agents[$i];
        my $count = $user_agents{$agent};
        my $display_agent = length($agent) > 60 ? substr($agent, 0, 57) . '...' : $agent;
        print $fh sprintf("%3d. [%4d] %s\n", $i + 1, $count, $display_agent);
    }
    
    print $fh "\n";
}

#==============================================================================
# Utility Functions
#==============================================================================

sub format_bytes {
    my ($bytes) = @_;
    
    my @units = qw(B KB MB GB TB);
    my $unit_index = 0;
    
    while ($bytes >= 1024 && $unit_index < $#units) {
        $bytes /= 1024;
        $unit_index++;
    }
    
    return sprintf("%.2f %s", $bytes, $units[$unit_index]);
}

sub create_sample_log_data {
    my $filename = 'sample_access.log';
    
    open my $fh, '>', $filename or die "Cannot create sample log file: $!";
    
    # Generate sample log entries
    my @sample_ips = qw(192.168.1.100 10.0.0.15 203.0.113.45 198.51.100.78 192.0.2.123);
    my @sample_urls = qw(/ /index.html /about.html /products.html /login.php /admin /api/data);
    my @sample_agents = (
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
        'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36'
    );
    my @status_codes = (200, 200, 200, 200, 404, 500, 301, 403);
    
    my $base_time = time() - 3600;  # 1 hour ago
    
    for my $i (0 .. 999) {  # Generate 1000 log entries
        my $ip = $sample_ips[rand(@sample_ips)];
        my $timestamp = $base_time + int(rand(3600));
        my $url = $sample_urls[rand(@sample_urls)];
        my $status = $status_codes[rand(@status_codes)];
        my $size = int(rand(50000)) + 1000;
        my $agent = $sample_agents[rand(@sample_agents)];
        
        my $timestamp_str = strftime('%d/%b/%Y:%H:%M:%S %z', localtime($timestamp));
        
        print $fh sprintf(
            "%s - - [%s] \"GET %s HTTP/1.1\" %d %d \"-\" \"%s\"\n",
            $ip, $timestamp_str, $url, $status, $size, $agent
        );
        
        # Add some suspicious entries
        if (rand() < 0.05) {  # 5% chance
            print $fh sprintf(
                "%s - - [%s] \"GET %s?id=' UNION SELECT * FROM users-- HTTP/1.1\" 200 1234 \"-\" \"sqlmap/1.0\"\n",
                $ip, $timestamp_str, $url
            );
        }
    }
    
    close $fh;
    return $filename;
}

#==============================================================================
# Main Program Logic
#==============================================================================

sub parse_command_line {
    GetOptions(
        'input|i=s'      => \$config{input_file},
        'output|o=s'     => \$config{output_file},
        'format|f=s'     => \$config{log_format},
        'window|w=i'     => \$config{time_window},
        'top|t=i'        => \$config{top_entries},
        'frequency|F=i'  => \$config{min_frequency},
        'charts|c'       => \$config{generate_charts},
        'verbose|v'      => \$config{verbose},
        'debug|d'        => \$DEBUG,
        'help|h'         => \$config{help},
    ) or die "Error parsing command line options!\n";
    
    if ($config{help}) {
        print_usage();
        exit 0;
    }
}

sub print_usage {
    print << "EOF";
$PROGRAM_NAME v$VERSION

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -i, --input FILE        Input log file (default: create sample data)
    -o, --output FILE       Output report file (default: STDOUT)
    -f, --format FORMAT     Log format (combined|common) [default: combined]
    -w, --window SECONDS    Time window for analysis [default: 3600]
    -t, --top NUMBER        Number of top entries to show [default: 10]
    -F, --frequency NUM     Minimum frequency threshold [default: 1]
    -c, --charts            Generate ASCII charts
    -v, --verbose           Verbose output
    -d, --debug             Debug mode
    -h, --help              Show this help message

EXAMPLES:
    $0 --input /var/log/apache2/access.log --output report.txt
    $0 --input access.log --top 20 --charts
    $0 --format common --window 7200

EOF
}

sub process_log_file {
    my ($filename) = @_;
    
    print STDERR "Processing log file: $filename\n" if $config{verbose};
    
    my $fh = IO::File->new($filename, 'r') 
        or die "Cannot open log file '$filename': $!\n";
    
    my $line_number = 0;
    
    while (my $line = <$fh>) {
        chomp $line;
        $line_number++;
        $total_lines_processed++;
        
        # Skip empty lines and comments
        next if $line =~ /^\s*$/ or $line =~ /^\s*#/;
        
        # Parse log entry based on format
        my $entry;
        if ($config{log_format} eq 'combined') {
            $entry = LogParser::parse_combined_format($line);
        } elsif ($config{log_format} eq 'common') {
            $entry = LogParser::parse_common_format($line);
        } else {
            die "Unsupported log format: $config{log_format}\n";
        }
        
        if (!$entry) {
            $parse_errors++;
            print STDERR "Parse error at line $line_number: $line\n" if $DEBUG;
            next;
        }
        
        # Store entry for analysis
        push @log_entries, $entry;
        
        # Update statistics
        update_statistics($entry);
        
        # Progress indicator
        if ($config{verbose} && $line_number % 10000 == 0) {
            print STDERR "Processed $line_number lines...\n";
        }
    }
    
    $fh->close();
    
    print STDERR "Finished processing $total_lines_processed lines\n" if $config{verbose};
}

sub update_statistics {
    my ($entry) = @_;
    
    # IP address statistics
    $ip_stats{$entry->{ip_address}}->{request_count}++;
    $ip_stats{$entry->{ip_address}}->{bytes_sent} += $entry->{response_size};
    
    if ($entry->is_error()) {
        $ip_stats{$entry->{ip_address}}->{error_count}++;
    }
    
    if ($entry->is_suspicious()) {
        $ip_stats{$entry->{ip_address}}->{suspicious_requests}++;
    }
    
    # Calculate error rate
    my $total_requests = $ip_stats{$entry->{ip_address}}->{request_count};
    my $error_count = $ip_stats{$entry->{ip_address}}->{error_count} || 0;
    $ip_stats{$entry->{ip_address}}->{error_rate} = $error_count / $total_requests;
    
    # URL statistics
    $url_stats{$entry->{url}}++;
    
    # Status code statistics
    $status_codes{$entry->{status_code}}++;
    
    # User agent statistics
    $user_agents{$entry->{user_agent}}++;
    
    # Referrer statistics
    $referrers{$entry->{referrer}}++ if $entry->{referrer};
    
    # Time-based statistics
    my $hour = $entry->get_hour();
    my $date = $entry->get_date();
    
    $hourly_traffic{$hour}++;
    $daily_traffic{$date}++;
    
    # Security event tracking
    if ($entry->is_suspicious()) {
        push @{$security_events{$entry->{ip_address}}}, $entry;
    }
}

sub main {
    print "="x80 . "\n";
    print "$PROGRAM_NAME v$VERSION\n";
    print "Perl-based Log Analysis and Security Monitoring Tool\n";
    print "="x80 . "\n\n";
    
    # Parse command line arguments
    parse_command_line();
    
    # Determine input file
    my $input_file = $config{input_file};
    if (!$input_file) {
        print "No input file specified. Creating sample log data...\n";
        $input_file = create_sample_log_data();
        print "Sample log file created: $input_file\n\n";
    }
    
    # Process the log file
    process_log_file($input_file);
    
    # Open output file or use STDOUT
    my $output_fh;
    if ($config{output_file}) {
        $output_fh = IO::File->new($config{output_file}, 'w')
            or die "Cannot create output file '$config{output_file}': $!\n";
        print "Generating report: $config{output_file}\n\n";
    } else {
        $output_fh = \*STDOUT;
    }
    
    # Generate comprehensive report
    generate_comprehensive_report($output_fh);
    
    # Close output file
    if ($config{output_file}) {
        $output_fh->close();
        print "\nReport generation completed!\n";
    }
    
    # Display summary
    print "\n" . "="x80 . "\n";
    print "PROCESSING SUMMARY\n";
    print "-"x80 . "\n";
    print "Total entries processed: $total_lines_processed\n";
    print "Parse errors: $parse_errors\n";
    print "Unique IP addresses: " . (keys %ip_stats) . "\n";
    print "Unique URLs: " . (keys %url_stats) . "\n";
    print "Processing time: " . (time() - $processing_start_time) . " seconds\n";
    
    # Security summary
    my $suspicious_ips = analyze_ip_patterns();
    my $attacks = detect_attack_patterns();
    my $total_attacks = sum(values %$attacks);
    
    print "\nSECURITY SUMMARY\n";
    print "-"x80 . "\n";
    print "Suspicious IP addresses: " . @$suspicious_ips . "\n";
    print "Potential attack attempts: $total_attacks\n";
    
    for my $attack_type (keys %$attacks) {
        my $count = $attacks->{$attack_type};
        print "  " . ucfirst($attack_type) . ": $count\n" if $count > 0;
    }
    
    print "\nDemonstrated Perl features:\n";
    print "- Regular expressions and pattern matching\n";
    print "- Object-oriented programming with packages\n";
    print "- Complex data structures (hashes, arrays, references)\n";
    print "- File I/O and text processing\n";
    print "- Statistical analysis and calculations\n";
    print "- Command-line argument processing\n";
    print "- Date/time manipulation\n";
    print "- Advanced Perl idioms and best practices\n";
    print "- Performance optimization techniques\n";
    print "- Error handling and debugging\n";
    
    # Cleanup sample file if created
    if (!$config{input_file} && -f 'sample_access.log') {
        unlink 'sample_access.log';
        print "\nCleaned up sample log file\n";
    }
}

# Run the main program
main() unless caller;

# End of program
1;

__END__

=head1 NAME

log_analyzer.pl - Comprehensive Log Analysis System

=head1 SYNOPSIS

    perl log_analyzer.pl [options]

=head1 DESCRIPTION

This program demonstrates comprehensive log analysis capabilities using advanced
Perl programming techniques. It processes web server log files and generates
detailed security and performance reports.

=head1 FEATURES

- Multiple log format support (Apache Combined/Common)
- Real-time log parsing and analysis
- Statistical analysis with percentiles
- Security threat detection
- Performance monitoring
- Traffic pattern analysis
- Comprehensive reporting

=head1 AUTHOR

Rigr Test Suite - Perl Language Demonstration

=head1 VERSION

1.0.0

=cut