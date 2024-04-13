#!/home/dwhitfield/Developer/Artisan_Hosting/Artisan_Interactivity_System/target/debug/ais_python

import ais

version = ais.version()
enc_test = ais.encrypt_text("Hello from ais")

if enc_test != None:
    dec_test = ais.decrypt_text(enc_test)

    if dec_test != None:
        data = bytes.fromhex(f"{dec_test}").decode('utf-8')
        print(data)
        ais.send_email("The decrypted data", data)
    else:
        print("Decrypt failed")

else:
    print("Null returned")
    

print(version)
print(ais.get_hostname())
ais.debug_print()
