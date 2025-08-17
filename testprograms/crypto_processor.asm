;
; Comprehensive Cryptographic Processor in x86-64 Assembly
;
; This program demonstrates various assembly programming concepts including:
; - CPU register manipulation and optimization
; - Memory addressing modes and data structures
; - Arithmetic and bitwise operations
; - Loop constructs and conditional branching
; - Function calls and stack frame management
; - String processing and manipulation
; - Bit manipulation and cryptographic algorithms
; - System calls and I/O operations
; - Macro definitions and code generation
; - Performance optimization techniques
;

section .data
    ; Program information
    program_title db 'Comprehensive Cryptographic Processor v1.0', 0
    author_info db 'Assembly Language Demonstration Program', 0
    
    ; Menu and user interface strings
    main_menu db 10, '=== Cryptographic Processor Main Menu ===', 10
             db '1. Caesar Cipher Encryption/Decryption', 10
             db '2. XOR Cipher Operations', 10
             db '3. Base64 Encoding/Decoding', 10
             db '4. Hash Function (Simple)', 10
             db '5. Bit Manipulation Utilities', 10
             db '6. Random Number Generation', 10
             db '7. Performance Benchmarks', 10
             db '8. Memory Operations Test', 10
             db '9. String Analysis Tools', 10
             db '0. Exit Program', 10
             db 'Choice: ', 0
    
    ; Input/Output strings
    input_prompt db 'Enter text to process: ', 0
    key_prompt db 'Enter key/shift value: ', 0
    output_header db 10, 'Result: ', 0
    error_msg db 'Error: Invalid input or operation failed', 10, 0
    success_msg db 'Operation completed successfully', 10, 0
    
    ; Test data for demonstrations
    test_string db 'Hello, World! This is a test of assembly language programming.', 0
    test_key db 'SECRETKEY', 0
    alphabet db 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 0
    base64_chars db 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', 0
    
    ; Cryptographic tables and constants
    substitution_table times 256 db 0
    frequency_table times 26 dd 0
    random_seed dd 12345
    
    ; Buffer sizes and limits
    MAX_INPUT_SIZE equ 1024
    BUFFER_SIZE equ 2048
    
    ; Performance counters
    cycle_counter_start dq 0
    cycle_counter_end dq 0
    operation_count dd 0

section .bss
    ; Dynamic buffers for processing
    input_buffer resb MAX_INPUT_SIZE
    output_buffer resb BUFFER_SIZE
    temp_buffer resb BUFFER_SIZE
    key_buffer resb 256
    
    ; Working variables
    user_choice resb 4
    shift_value resb 4
    string_length resd 1
    hash_result resd 1
    
    ; Performance measurement variables
    benchmark_results resd 10

section .text
    global _start

; Macro definitions for common operations
%macro PRINT_STRING 1
    mov rdi, 1          ; stdout
    mov rsi, %1         ; string address
    call print_string
%endmacro

%macro READ_INPUT 2
    mov rdi, 0          ; stdin
    mov rsi, %1         ; buffer address
    mov rdx, %2         ; max length
    call read_input
%endmacro

%macro START_TIMER 0
    rdtsc
    mov [cycle_counter_start], rax
%endmacro

%macro END_TIMER 0
    rdtsc
    mov [cycle_counter_end], rax
%endmacro

; Main program entry point
_start:
    ; Initialize program
    call initialize_system
    
    ; Display program header
    PRINT_STRING program_title
    call print_newline
    PRINT_STRING author_info
    call print_newline
    
main_loop:
    ; Display main menu
    PRINT_STRING main_menu
    
    ; Get user choice
    READ_INPUT user_choice, 4
    
    ; Parse choice and branch
    mov al, [user_choice]
    sub al, '0'
    
    cmp al, 0
    je exit_program
    cmp al, 1
    je caesar_cipher_menu
    cmp al, 2
    je xor_cipher_menu
    cmp al, 3
    je base64_menu
    cmp al, 4
    je hash_function_menu
    cmp al, 5
    je bit_manipulation_menu
    cmp al, 6
    je random_number_menu
    cmp al, 7
    je performance_benchmark_menu
    cmp al, 8
    je memory_operations_menu
    cmp al, 9
    je string_analysis_menu
    
    ; Invalid choice
    PRINT_STRING error_msg
    jmp main_loop

; Caesar Cipher Implementation
caesar_cipher_menu:
    PRINT_STRING input_prompt
    READ_INPUT input_buffer, MAX_INPUT_SIZE
    
    PRINT_STRING key_prompt
    READ_INPUT shift_value, 4
    
    ; Convert shift value from ASCII
    mov al, [shift_value]
    sub al, '0'
    mov [shift_value], al
    
    ; Perform Caesar cipher encryption
    call caesar_encrypt
    
    ; Display result
    PRINT_STRING output_header
    PRINT_STRING output_buffer
    call print_newline
    
    jmp main_loop

; Caesar cipher encryption function
caesar_encrypt:
    push rbp
    mov rbp, rsp
    push rsi
    push rdi
    push rcx
    push rax
    push rdx
    
    mov rsi, input_buffer       ; source
    mov rdi, output_buffer      ; destination
    mov cl, [shift_value]       ; shift amount
    
caesar_encrypt_loop:
    mov al, [rsi]              ; load character
    test al, al                ; check for null terminator
    jz caesar_encrypt_done
    
    ; Check if character is alphabetic
    cmp al, 'A'
    jb caesar_encrypt_skip
    cmp al, 'Z'
    jbe caesar_encrypt_upper
    cmp al, 'a'
    jb caesar_encrypt_skip
    cmp al, 'z'
    ja caesar_encrypt_skip
    
    ; Handle lowercase letters
    sub al, 'a'                ; convert to 0-25
    add al, cl                 ; add shift
    mov dl, 26
    div dl                     ; modulo 26
    mov al, ah                 ; remainder to AL
    add al, 'a'                ; convert back to ASCII
    jmp caesar_encrypt_store
    
caesar_encrypt_upper:
    ; Handle uppercase letters
    sub al, 'A'                ; convert to 0-25
    add al, cl                 ; add shift
    mov dl, 26
    div dl                     ; modulo 26
    mov al, ah                 ; remainder to AL
    add al, 'A'                ; convert back to ASCII
    jmp caesar_encrypt_store
    
caesar_encrypt_skip:
    ; Non-alphabetic characters remain unchanged
    
caesar_encrypt_store:
    mov [rdi], al              ; store encrypted character
    inc rsi                    ; next source character
    inc rdi                    ; next destination position
    jmp caesar_encrypt_loop
    
caesar_encrypt_done:
    mov byte [rdi], 0          ; null terminate result
    
    pop rdx
    pop rax
    pop rcx
    pop rdi
    pop rsi
    pop rbp
    ret

; XOR Cipher Implementation
xor_cipher_menu:
    PRINT_STRING input_prompt
    READ_INPUT input_buffer, MAX_INPUT_SIZE
    
    ; Use predefined key for demo
    call xor_encrypt_decrypt
    
    PRINT_STRING output_header
    PRINT_STRING output_buffer
    call print_newline
    
    jmp main_loop

; XOR encryption/decryption function
xor_encrypt_decrypt:
    push rbp
    mov rbp, rsp
    push rsi
    push rdi
    push rcx
    push rax
    push rdx
    
    mov rsi, input_buffer       ; source text
    mov rdi, output_buffer      ; destination
    mov rcx, test_key           ; encryption key
    xor rdx, rdx               ; key index
    
xor_cipher_loop:
    mov al, [rsi]              ; load character
    test al, al                ; check for end
    jz xor_cipher_done
    
    ; Get key character (cycling through key)
    mov bl, [rcx + rdx]        ; load key character
    test bl, bl                ; check if end of key
    jnz xor_cipher_apply
    xor rdx, rdx               ; reset key index
    mov bl, [rcx]              ; first key character
    
xor_cipher_apply:
    xor al, bl                 ; XOR with key character
    mov [rdi], al              ; store result
    
    inc rsi                    ; next source character
    inc rdi                    ; next destination
    inc rdx                    ; next key position
    jmp xor_cipher_loop
    
xor_cipher_done:
    mov byte [rdi], 0          ; null terminate
    
    pop rdx
    pop rax
    pop rcx
    pop rdi
    pop rsi
    pop rbp
    ret

; Base64 Encoding Implementation
base64_menu:
    PRINT_STRING input_prompt
    READ_INPUT input_buffer, MAX_INPUT_SIZE
    
    call base64_encode
    
    PRINT_STRING output_header
    PRINT_STRING output_buffer
    call print_newline
    
    jmp main_loop

; Base64 encoding function
base64_encode:
    push rbp
    mov rbp, rsp
    push rsi
    push rdi
    push rax
    push rbx
    push rcx
    push rdx
    
    mov rsi, input_buffer       ; source
    mov rdi, output_buffer      ; destination
    
base64_encode_loop:
    ; Load 3 bytes (24 bits) from input
    mov eax, 0
    mov al, [rsi]              ; first byte
    test al, al
    jz base64_encode_done
    shl eax, 8
    
    inc rsi
    mov bl, [rsi]              ; second byte
    test bl, bl
    jz base64_encode_padding2
    mov al, bl
    shl eax, 8
    
    inc rsi
    mov bl, [rsi]              ; third byte
    test bl, bl
    jz base64_encode_padding1
    mov al, bl
    inc rsi
    
    ; Extract 4 6-bit groups from 24 bits
    mov ebx, eax               ; copy for manipulation
    
    ; First 6-bit group (bits 18-23)
    shr ebx, 18
    and ebx, 63
    mov cl, [base64_chars + rbx]
    mov [rdi], cl
    inc rdi
    
    ; Second 6-bit group (bits 12-17)
    mov ebx, eax
    shr ebx, 12
    and ebx, 63
    mov cl, [base64_chars + rbx]
    mov [rdi], cl
    inc rdi
    
    ; Third 6-bit group (bits 6-11)
    mov ebx, eax
    shr ebx, 6
    and ebx, 63
    mov cl, [base64_chars + rbx]
    mov [rdi], cl
    inc rdi
    
    ; Fourth 6-bit group (bits 0-5)
    mov ebx, eax
    and ebx, 63
    mov cl, [base64_chars + rbx]
    mov [rdi], cl
    inc rdi
    
    jmp base64_encode_loop
    
base64_encode_padding1:
    ; Handle single padding character
    mov byte [rdi], '='
    inc rdi
    jmp base64_encode_done
    
base64_encode_padding2:
    ; Handle double padding characters
    mov byte [rdi], '='
    inc rdi
    mov byte [rdi], '='
    inc rdi
    
base64_encode_done:
    mov byte [rdi], 0          ; null terminate
    
    pop rdx
    pop rcx
    pop rbx
    pop rax
    pop rdi
    pop rsi
    pop rbp
    ret

; Simple Hash Function Implementation
hash_function_menu:
    PRINT_STRING input_prompt
    READ_INPUT input_buffer, MAX_INPUT_SIZE
    
    call simple_hash
    
    ; Convert hash result to hex string for display
    call convert_hash_to_hex
    
    PRINT_STRING output_header
    PRINT_STRING output_buffer
    call print_newline
    
    jmp main_loop

; Simple hash function (djb2 algorithm)
simple_hash:
    push rbp
    mov rbp, rsp
    push rsi
    push rax
    push rbx
    
    mov rsi, input_buffer       ; input string
    mov eax, 5381               ; hash = 5381
    
hash_loop:
    mov bl, [rsi]              ; load character
    test bl, bl                ; check for end
    jz hash_done
    
    ; hash = hash * 33 + c
    imul eax, 33               ; multiply by 33
    movzx rbx, bl              ; zero-extend character
    add eax, ebx               ; add character
    
    inc rsi                    ; next character
    jmp hash_loop
    
hash_done:
    mov [hash_result], eax     ; store result
    
    pop rbx
    pop rax
    pop rsi
    pop rbp
    ret

; Convert hash to hexadecimal string
convert_hash_to_hex:
    push rbp
    mov rbp, rsp
    push rax
    push rbx
    push rcx
    push rdx
    push rdi
    
    mov eax, [hash_result]     ; load hash value
    mov rdi, output_buffer     ; destination
    mov ecx, 8                 ; 8 hex digits (32-bit value)
    
hex_convert_loop:
    rol eax, 4                 ; rotate left 4 bits
    mov bl, al                 ; copy low 4 bits
    and bl, 0x0F               ; mask to get nibble
    
    ; Convert to hex character
    cmp bl, 10
    jb hex_digit
    add bl, 'A' - 10
    jmp hex_store
    
hex_digit:
    add bl, '0'
    
hex_store:
    mov [rdi], bl
    inc rdi
    dec ecx
    jnz hex_convert_loop
    
    mov byte [rdi], 0          ; null terminate
    
    pop rdi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    pop rbp
    ret

; Bit Manipulation Utilities
bit_manipulation_menu:
    ; Demonstrate various bit operations
    mov eax, 0x12345678        ; test value
    
    ; Count set bits (population count)
    call count_set_bits
    
    ; Find first set bit
    call find_first_set_bit
    
    ; Reverse bits
    call reverse_bits
    
    ; Display results
    call display_bit_results
    
    jmp main_loop

; Count set bits in EAX
count_set_bits:
    push rbp
    mov rbp, rsp
    push rbx
    push rcx
    
    xor ebx, ebx               ; bit counter
    mov ecx, 32                ; 32 bits to check
    
count_bits_loop:
    shr eax, 1                 ; shift right
    adc ebx, 0                 ; add carry to counter
    dec ecx
    jnz count_bits_loop
    
    ; Store result for display
    mov [temp_buffer], ebx
    
    pop rcx
    pop rbx
    pop rbp
    ret

; Find first set bit (returns position in EBX, 0-based)
find_first_set_bit:
    push rbp
    mov rbp, rsp
    push rcx
    
    bsf ebx, eax               ; bit scan forward
    jnz first_bit_found
    mov ebx, -1                ; no bits set
    
first_bit_found:
    ; Store result
    mov [temp_buffer + 4], ebx
    
    pop rcx
    pop rbp
    ret

; Reverse bits in EAX
reverse_bits:
    push rbp
    mov rbp, rsp
    push rbx
    push rcx
    push rdx
    
    xor ebx, ebx               ; result
    mov ecx, 32                ; bit count
    
reverse_loop:
    shl ebx, 1                 ; shift result left
    shr eax, 1                 ; shift input right
    adc ebx, 0                 ; add carry to result
    dec ecx
    jnz reverse_loop
    
    mov [temp_buffer + 8], ebx ; store result
    
    pop rdx
    pop rcx
    pop rbx
    pop rbp
    ret

; Display bit manipulation results
display_bit_results:
    ; Implementation would format and display the results
    ; Simplified for this demonstration
    PRINT_STRING success_msg
    ret

; Random Number Generation
random_number_menu:
    ; Generate sequence of random numbers
    mov ecx, 10                ; generate 10 numbers
    
random_gen_loop:
    call linear_congruential_generator
    call display_random_number
    dec ecx
    jnz random_gen_loop
    
    jmp main_loop

; Linear Congruential Generator
linear_congruential_generator:
    push rbp
    mov rbp, rsp
    push rax
    push rdx
    
    ; X(n+1) = (a * X(n) + c) mod m
    ; Using: a = 1664525, c = 1013904223, m = 2^32
    mov eax, [random_seed]
    imul eax, 1664525          ; multiply by a
    add eax, 1013904223        ; add c
    ; mod 2^32 is automatic due to 32-bit register overflow
    
    mov [random_seed], eax     ; update seed
    
    pop rdx
    pop rax
    pop rbp
    ret

; Display random number (simplified)
display_random_number:
    ; Convert to string and display
    ; Simplified implementation
    ret

; Performance Benchmark Suite
performance_benchmark_menu:
    call benchmark_arithmetic_operations
    call benchmark_memory_operations
    call benchmark_string_operations
    call display_benchmark_results
    jmp main_loop

; Arithmetic operations benchmark
benchmark_arithmetic_operations:
    START_TIMER
    
    mov ecx, 1000000           ; iteration count
    mov eax, 1
    mov ebx, 2
    
arithmetic_benchmark_loop:
    add eax, ebx               ; addition
    imul eax, 3                ; multiplication
    shr eax, 1                 ; division by 2
    dec ecx
    jnz arithmetic_benchmark_loop
    
    END_TIMER
    
    ; Calculate elapsed cycles
    mov rax, [cycle_counter_end]
    sub rax, [cycle_counter_start]
    mov [benchmark_results], eax
    
    ret

; Memory operations benchmark
benchmark_memory_operations:
    START_TIMER
    
    mov ecx, 100000            ; iteration count
    mov rsi, input_buffer      ; source
    mov rdi, output_buffer     ; destination
    
memory_benchmark_loop:
    ; Copy memory block
    push rcx
    mov ecx, 256               ; bytes to copy
    rep movsb                  ; string move
    pop rcx
    
    ; Reset pointers
    mov rsi, input_buffer
    mov rdi, output_buffer
    
    dec ecx
    jnz memory_benchmark_loop
    
    END_TIMER
    
    mov rax, [cycle_counter_end]
    sub rax, [cycle_counter_start]
    mov [benchmark_results + 4], eax
    
    ret

; String operations benchmark
benchmark_string_operations:
    START_TIMER
    
    mov ecx, 50000             ; iteration count
    
string_benchmark_loop:
    ; String length calculation
    mov rsi, test_string
    call string_length_calc
    
    ; String comparison
    mov rsi, test_string
    mov rdi, test_string
    call string_compare
    
    dec ecx
    jnz string_benchmark_loop
    
    END_TIMER
    
    mov rax, [cycle_counter_end]
    sub rax, [cycle_counter_start]
    mov [benchmark_results + 8], eax
    
    ret

; String length calculation
string_length_calc:
    push rdi
    push rcx
    
    mov rdi, rsi               ; string address
    xor rcx, rcx               ; length counter
    
strlen_loop:
    cmp byte [rdi], 0          ; check for null terminator
    je strlen_done
    inc rdi
    inc rcx
    jmp strlen_loop
    
strlen_done:
    mov [string_length], ecx
    
    pop rcx
    pop rdi
    ret

; String comparison
string_compare:
    push rax
    push rbx
    
string_cmp_loop:
    mov al, [rsi]              ; load from first string
    mov bl, [rdi]              ; load from second string
    cmp al, bl                 ; compare characters
    jne string_cmp_done        ; not equal
    test al, al                ; check for end
    jz string_cmp_done         ; end reached
    inc rsi
    inc rdi
    jmp string_cmp_loop
    
string_cmp_done:
    pop rbx
    pop rax
    ret

; Display benchmark results
display_benchmark_results:
    PRINT_STRING success_msg
    ret

; Memory Operations Test
memory_operations_menu:
    call test_memory_copy
    call test_memory_set
    call test_memory_search
    jmp main_loop

; Test memory copy operations
test_memory_copy:
    ; Copy test string to temp buffer
    mov rsi, test_string
    mov rdi, temp_buffer
    mov rcx, 64                ; bytes to copy
    rep movsb
    
    PRINT_STRING success_msg
    ret

; Test memory set operations
test_memory_set:
    ; Fill buffer with pattern
    mov rdi, temp_buffer + 100
    mov al, 0xAA               ; fill pattern
    mov rcx, 100               ; bytes to fill
    rep stosb
    
    ret

; Test memory search operations
test_memory_search:
    ; Search for character in buffer
    mov rdi, test_string
    mov al, 'W'                ; character to find
    mov rcx, 64                ; search length
    repne scasb                ; scan until found or end
    
    ret

; String Analysis Tools
string_analysis_menu:
    PRINT_STRING input_prompt
    READ_INPUT input_buffer, MAX_INPUT_SIZE
    
    call analyze_string_composition
    call calculate_frequency_distribution
    call find_palindromes
    
    PRINT_STRING success_msg
    jmp main_loop

; Analyze string composition
analyze_string_composition:
    push rsi
    push rax
    push rbx
    push rcx
    
    mov rsi, input_buffer
    xor eax, eax               ; character counters
    xor ebx, ebx
    xor ecx, ecx
    
analyze_loop:
    mov dl, [rsi]
    test dl, dl
    jz analyze_done
    
    ; Count different character types
    cmp dl, 'A'
    jb analyze_next
    cmp dl, 'Z'
    jbe analyze_upper
    cmp dl, 'a'
    jb analyze_next
    cmp dl, 'z'
    jbe analyze_lower
    jmp analyze_next
    
analyze_upper:
    inc eax                    ; uppercase count
    jmp analyze_next
    
analyze_lower:
    inc ebx                    ; lowercase count
    
analyze_next:
    inc esi
    jmp analyze_loop
    
analyze_done:
    ; Store results
    mov [temp_buffer], eax
    mov [temp_buffer + 4], ebx
    
    pop rcx
    pop rbx
    pop rax
    pop rsi
    ret

; Calculate character frequency distribution
calculate_frequency_distribution:
    push rsi
    push rax
    push rbx
    push rcx
    
    ; Clear frequency table
    mov rdi, frequency_table
    xor eax, eax
    mov ecx, 26
    rep stosd
    
    ; Count character frequencies
    mov rsi, input_buffer
    
freq_loop:
    mov al, [rsi]
    test al, al
    jz freq_done
    
    ; Convert to uppercase and check range
    cmp al, 'a'
    jb freq_check_upper
    cmp al, 'z'
    ja freq_next
    sub al, 'a' - 'A'          ; convert to uppercase
    
freq_check_upper:
    cmp al, 'A'
    jb freq_next
    cmp al, 'Z'
    ja freq_next
    
    ; Increment frequency counter
    sub al, 'A'                ; convert to index (0-25)
    movzx rbx, al
    inc dword [frequency_table + rbx * 4]
    
freq_next:
    inc rsi
    jmp freq_loop
    
freq_done:
    pop rcx
    pop rbx
    pop rax
    pop rsi
    ret

; Find palindromes (simplified)
find_palindromes:
    ; Implementation would check for palindromic substrings
    ; Simplified for this demonstration
    ret

; System utility functions

; Initialize system and data structures
initialize_system:
    push rax
    push rcx
    push rdi
    
    ; Initialize substitution table with identity mapping
    mov rdi, substitution_table
    xor eax, eax
    mov ecx, 256
    
init_sub_table:
    mov [rdi], al
    inc rdi
    inc eax
    dec ecx
    jnz init_sub_table
    
    ; Initialize frequency table
    mov rdi, frequency_table
    xor eax, eax
    mov ecx, 26
    rep stosd
    
    pop rdi
    pop rcx
    pop rax
    ret

; Print string function (simplified system call interface)
print_string:
    push rax
    push rdi
    push rsi
    push rdx
    
    ; Calculate string length
    mov rdi, rsi               ; string address
    xor rdx, rdx               ; length counter
    
print_strlen_loop:
    cmp byte [rdi + rdx], 0    ; check for null terminator
    je print_strlen_done
    inc rdx
    jmp print_strlen_loop
    
print_strlen_done:
    ; System call to write
    mov rax, 1                 ; sys_write
    mov rdi, 1                 ; stdout
    ; rsi already contains string address
    ; rdx contains length
    syscall
    
    pop rdx
    pop rsi
    pop rdi
    pop rax
    ret

; Read input function (simplified)
read_input:
    push rax
    push rdi
    push rsi
    push rdx
    
    mov rax, 0                 ; sys_read
    ; rdi = file descriptor (already set)
    ; rsi = buffer address (already set)
    ; rdx = max bytes (already set)
    syscall
    
    ; Null-terminate the input
    dec rax                    ; remove newline
    mov byte [rsi + rax], 0
    
    pop rdx
    pop rsi
    pop rdi
    pop rax
    ret

; Print newline
print_newline:
    push rax
    push rdi
    push rsi
    push rdx
    
    mov rax, 1                 ; sys_write
    mov rdi, 1                 ; stdout
    mov rsi, newline_char      ; newline character
    mov rdx, 1                 ; length
    syscall
    
    pop rdx
    pop rsi
    pop rdi
    pop rax
    ret

; Exit program
exit_program:
    PRINT_STRING success_msg
    
    mov rax, 60                ; sys_exit
    xor rdi, rdi               ; exit code 0
    syscall

; Additional data
section .data
    newline_char db 10, 0

; Program ends here - total demonstration of:
; - Register manipulation and optimization
; - Memory addressing and data structures
; - Arithmetic and bitwise operations
; - Loop constructs and branching
; - Function calls and stack management
; - String processing algorithms
; - Cryptographic implementations
; - Performance measurement
; - System calls and I/O
; - Macro programming
; - Code organization and documentation