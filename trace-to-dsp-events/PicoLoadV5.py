# Version 1.1 DEP 2020.
# Code to read picscope .traces file, a binary file developed by E.M Schooneveld. 
# Code structure adapted from EMS's reader "to Text"
# Implemenets the same corrections (time and offsets) as in EMS's code.
# Vesrion 1 reads loads the data into a numpy array (3D, being channels x traces x events ).  This is initialised with zeros and populates using python slices.
# Version 1.1 first populates a list of lists using python .append as this benchmarked faster.
# EoF handinlg was tried using 1) a flag, 2) catching pythons EoF “”. This required two If statements (on read, and return of trace). 3) Using seek(0,2), and .tell to determine EoF when doing a file check. This requires one additional condition on while loop, and benchmarked fastest 
# Todo: Try Make entire thing a method, and the global variables .self 
# ToFutureMe: GUI.


import numpy as np
from scipy import stats
import matplotlib as mpl
import os
import matplotlib.pyplot as plt
from timeit import default_timer as timer
import sys
from pathlib import Path
import peakutils


def printme(dump_to_screen):
    LinesPlease()
    print(str(dump_to_screen))
  
# A simple function to print lines for pretty formatting
def LinesPlease(total = 1):
	i = 0
	while i < total:
		print("|------------------------------------------------------------------|")    
		i+=1

def FileExistChecker( file_location ):
    #Check the file exists, return the EoF byte index (some big integer) if okay
    #Close file here too.
   
    try:
        f= open(file_location,"rb")  
        f.read(1)                
        f.seek(0,2) #Jumps to the end
        EoFLocation = f.tell()    #Give you the end location (characters from start)
        f.close
        print("fileDir ",file_location, "was sucsessfully opened, read, and closed. Good to go! ")
        print("EoFLocation =  ", EoFLocation)
        LinesPlease()
    except OSError as err:
        print("OS error: {0}, ".format(err), "Check filename!")
        return 0
    except:
        print("Unexpected error:", sys.exc_info()[0]) 
        return 0
    
    #print("Time to run FileExistChecker                = ", timer() - start)
    #print("file_location                               = ", file_location)    

    return EoFLocation

      
def ReadHeader( f ):
    #Some variables are made global as they are correction to be applied during readEvent()
    global NbrChannels, NbrChanEnab, NbrSamples, VoltsScaleFact, ChanOffsetVolts, SampInterval, HeaderNamelist, HeaderValuelist

    # two lists that hold all info read from header sepearatle
    # initalise with titles, will be saved out later. 
    HeaderNamelist = ["Opened with"]
    HeaderValuelist = ["DEP_PicoReader_v1"]


    #HeaderNamelist.append("Opened with")
    #HeaderValuelist.append("DEP_PicoReader_v1")
    #Open and read header info from .traces file.         

    try:
        #Read the header of the .traces file. For any flexible firelds (like version, user comment or number of channels )
        # you first read a integer which gives the length of the field. 
        #
        #Read the version of the PicoScope program that has saved the file.
        #First read length of version string (Int32) 
        progVersionStrLen = np.fromfile(f, dtype='<i4', count=1) # fromfile will read count 'lots' of dype, and return each as an element in an array
        HeaderNamelist.append("progVersionStrLen")
        HeaderValuelist.append(progVersionStrLen[0])
                 
        tmpChars = np.fromfile(f, dtype='b', count=progVersionStrLen[0]) # i1 or b for byte will work 
        ProgVersionStr = ''
        for character in tmpChars: # loop over binary array to convert to ascii char and then string to append together.
            ProgVersionStr += str(chr(character)) 
        HeaderNamelist.append("ProgVersionStr")
        HeaderValuelist.append(ProgVersionStr)           
                   
        # Read the comment that user has entered as description of the run.
        tmpArray = np.fromfile(f, dtype='<i4', count=1)
        runDescriptStrLen = tmpArray[0]
        HeaderNamelist.append("runDescriptStrLen")
        HeaderValuelist.append(runDescriptStrLen)        
           
        tmpChars = np.fromfile(f, dtype='b', count=runDescriptStrLen)
        RunDescriptStr = ''
        for character in tmpChars: # loop over binary array to convert to ascii char and then string to append together.
            RunDescriptStr += str(chr(character)) 
        HeaderNamelist.append("RunDescriptStr")
        HeaderValuelist.append(RunDescriptStr)     

        #This is (usually) a dummy parameter and is intended only for information.
        tmpArray = np.fromfile(f, dtype='<i4', count=1)
        Resolution = tmpArray[0]
        HeaderNamelist.append("Resolution")
        HeaderValuelist.append(Resolution)
 
        #Read the number of channels of this PicoScope. Enabled value for channel A is written first.
        tmpArray = np.fromfile(f, dtype='<i4', count=1)
        NbrChannels = tmpArray[0]
        HeaderNamelist.append("NbrChannels")
        HeaderValuelist.append(NbrChannels)
        
        #Read whether each channel is enabled or not. Enabled value for channel A is written first.
        ChanEnabled = np.fromfile(f, dtype='bool', count=NbrChannels)
        #Could have added np array of values as list item, but chose to append each individually for ease later
        for i in range(NbrChannels):
            HeaderNamelist.append("ChanEnabled[" + str(i) + "]")
            HeaderValuelist.append(ChanEnabled[i])            

        # wanted a global int with the number fo good data channels.Avoids accidently using the number of channels the picscope has, or repeatedly summing the bool array. 
        NbrChanEnab = sum(ChanEnabled)

        # For all channels, read multiplication factor to convert (ADC) Int16 data to double.
        VoltsScaleFact = np.fromfile(f, dtype='float64', count=NbrChannels)
        for i in range(NbrChannels):
            HeaderNamelist.append("VoltsScaleFact[" + str(i) + "]")
            HeaderValuelist.append(VoltsScaleFact[i])

        #Read offsets in volts. 
        #In analysis code you have to subtract this offset from the volts obtained from multiplication factor.        
        ChanOffsetVolts = np.fromfile(f, dtype='float64', count=NbrChannels)
        for i in range(NbrChannels):
            HeaderNamelist.append("ChanOffsetVolts[" + str(i) + "]")
            HeaderValuelist.append(ChanOffsetVolts[i])
        
        #Read sample time in seconds and the number of samples. The delay time can change from event to event, so saved in event header instead of here (file header).
        tmpArray = np.fromfile(f, dtype='float64', count=1) #sample time in seconds (Double).
        SampInterval = tmpArray[0]
        HeaderNamelist.append("SampInterval")
        HeaderValuelist.append(SampInterval)       
        
        #Number of samples (Int32)
        tmpArray = np.fromfile(f, dtype='<i4', count=1)       
        NbrSamples = tmpArray[0]
        HeaderNamelist.append("NbrSamples")
        HeaderValuelist.append(NbrSamples)    
        
        #For all channels, read whether the trigger is enabled, the trigger levels (in Volts) and trigger slopes.
        #Values for external trigger are read after trigger values for channels.
        #Trigger slope: 0 = Above, 1 = Below, 2 = Rising, 3 = Falling, 4 = RisingOrFalling. Other trigger modes are not (yet) supported.
        
        TriggerEnabled = np.fromfile(f, dtype='bool', count=NbrChannels)
        for i in range(NbrChannels):
            HeaderNamelist.append("TriggerEnabled[" + str(i) + "]")
            HeaderValuelist.append(TriggerEnabled[i])       
        
        #Different to Erik's code- he asignes this to the end of array. Here I just read it with its own name, and construct same parameter name for header writeout.
        tmpArray = np.fromfile(f, dtype='bool', count=1)
        ExTriggerEnabled = tmpArray[0]
        #print("ExTriggerEnabled = ", ExTriggerEnabled)
        HeaderNamelist.append("TriggerEnabled[" + str(NbrChannels) + "]") # the external is saved as the last channel number (usually 4). NOT external trigger, to be consitent with Erik
        HeaderValuelist.append(ExTriggerEnabled)              
        
        TriggerLevel = np.fromfile(f, dtype='float64', count=NbrChannels)
        #print("TriggerLevel = ", TriggerLevel)
        for i in range(NbrChannels):
            HeaderNamelist.append("TriggerLevel[" + str(i) + "]")
            HeaderValuelist.append(TriggerLevel[i])  
        
        tmpArray = np.fromfile(f, dtype='float64', count=1)
        ExTriggerLevel = tmpArray[0]
        #print("ExTriggerLevel = ", ExTriggerLevel)
        HeaderNamelist.append("TriggerLevel[" + str(NbrChannels) + "]") # the external is saved as the last channel number (usually 4)
        HeaderValuelist.append(ExTriggerLevel)
        
        
        TriggerSlope = np.fromfile(f, dtype='<i4', count=NbrChannels)
        for i in range(NbrChannels):
            HeaderNamelist.append("TriggerSlope[" + str(i) + "]")
            HeaderValuelist.append(TriggerSlope[i]) 
                
        tmpArray = np.fromfile(f, dtype='<i4', count=1)
        ExTriggerSlope = tmpArray[0] 
        #print("ExTriggerSlope = ", ExTriggerSlope)
        HeaderNamelist.append("TriggerSlope[" + str(NbrChannels) + "]") # the external is saved as the last channel number (usually 4)
        HeaderValuelist.append(ExTriggerSlope)                
            

    except:
        print("Unexpected error, caught in  function ReadHeader :", sys.exc_info()[0]) 
        print("File handle ",f )
        printme("^ ---- ERROR --- ^ ")           
    
    #print("Time to ReadHeader                          = ", timer() - start)
    #print("fileDir                                    = ",fileDir+filename)     
        
    return True # Return the number of event in this file


def ReadEvent(f):
    #Have NbrChannels,NbrChanEnab , NbrSamples, VoltsScaleFact, ChanOffsetVolts, SampInterval as Globals
    #https://stackoverflow.com/questions/519633/lazy-method-for-reading-big-file-in-python
    #"Run after the header has been read, so file stream in right place "
    global triggerTime, triggerTimes
    
    triggerTimes = []# store these as we might do a fine correction 
    while True:   
            
        #Read the index/count of current event.
        curEvent = np.fromfile(f, dtype='<i4', count=1)    
        #print("In Read Event curEvent = ", curEvent)
        #Read the run time of the event. Run time is given in seconds, so have to read Double type variable from disk.
        #Time elapsed since start of run, given in seconds (Double).
        eveRunTime = np.fromfile(f, dtype='float64', count=1)
        
        #Read number traces in this event that have been saved.
        nbrSavedTraces = np.fromfile(f, dtype='<i4', count=1)
        
        #Read, for all channels, whether its trace has been saved to disk.
        # I don't know why this would differ here from NbrChanEnab (the sum of the 'enabled' bool array in main header. Ask Erik? )
        # I am not going to use SavedChannels (yet) as it might be less than NbrChanEnab, making my Array indicies wrong. hmmm.     
        SavedChannels = np.fromfile(f, dtype='bool', count=NbrChannels) # 
        
        #Read the trigger time, which incorporates the trigger time offset to get a more accurate determination of the trigger time.
        #Trigger time is given in seconds and is time from first sample to the time the trigger occured; is therefore postive.
        #Trigger time offset is the same for all traces/channels of an event but will vary from event to event.
        #The offset is zero if the waveform crosses the threshold at the trigger sampling instant, or a positive or negative value if jitter correction is required.
        # Trigger time given in seconds from start of trace (Double)
        triggerTime = np.fromfile(f, dtype='float64', count=1)
        triggerTimes.extend(triggerTime)
        
    
        #I return one event, which will contain the traces for as many digitiser channels there are
        # However, if a channel is not True in the SavedChannels array, then it will be skipped and
        # the numpy row will remain zeros. 
        # I am not sure why SavedChannels and NbrChanEnab exitist, need to check with Erik.
        #oneEvent=np.zeros((NbrChannels,NbrSamples)) # Initalise the event array (2d) Fill with zeros. 
         # this is a python list not numpy array yet
       
        oneEvent = []        
        for i in range(NbrChanEnab): # itterate over the number of channels before returning the event array
            if SavedChannels[i]: # bool test, for the channels saved in the .traces binary
                rawtrace = np.fromfile(f, dtype='int16', count=(NbrSamples)) # read the binary (2 bytes)
                trace = (VoltsScaleFact[i] * rawtrace) - ChanOffsetVolts[i] # scale and offset
                oneEvent.append(trace)
        yield oneEvent  #use yield (not return) to keep all loop counters at same value
        # look at "generators" in python for more info 
         
    print("Am at end of generator, but should never get here as I should break out due to "" EOF or flag=false. Fault in code. ")
    yield False
    #this should never be met as EoF
        

#----------------------------------------------------------------------

def ReadChunk(filepath, eventChunk = False):
    # https://stackoverflow.com/questions/519633/lazy-method-for-reading-big-file-in-python
    # Run this after the header has been read, so file stream pointer in right place 
    global eventsum    
    
    eventID = 0 # counter for Event number (one event contains as many traces as enabled)
    eventsum = 0 # for total events read -  used when several chunks are read in

    f = open(filepath,"rb")  
    if eventID == 0: # if this is the first event in file, need to get headers first
        try:
            HeaderOKAY = ReadHeader(f) 
        except:
            print("Unexpected error:", sys.exc_info()[0]) 
            print("File should have existed, error comes when reading header. Boolean returned is = ", HeaderOKAY)
            yield 0
    
    # Create the readevent generator
    ReadEventGen = ReadEvent(f)
    # If no chunk size given, default to read whole file
    # Separate this case with one IF statement, as it makes the while condition one-fold, as no need to check if "chunk size" reached
    if eventChunk == False: # read whole file
        eventMatrix = []
        while f.tell() != EoFLocation:  #loop to read x events = 'a chunk'               
                event = next(ReadEventGen)  #returns a 2D array, channels x numbSamples     
                eventMatrix.append(event)   # in this version I append an event array to a list        
                eventID += 1
                eventsum = eventID
        
    # If a chunk size given, I need to run yeild when chunksize is reached  
    # The while loop still checks for EoF, which exhausts the generator when met. 
    # I break out of this (and close file) when EoF reached.
    else:   
        
        #print("TOP eventsum = ", eventsum, "eventID = ", eventID, "eventChunk = ", eventChunk)       
        eventID = 0  # need to reset this here, back to zero for the start of each chunk            
        eventMatrix = [] # Initalise the event matrix before next chunk read  
        
        while  f.tell() != EoFLocation: #loop to keep loading and appending events, but ends at EoF
        
            #print("in eventID loop ", eventID)
            event = next(ReadEventGen)  #returns a 2D array, channels x numbSamples        
            eventMatrix.append(event)   # in this version I append an event array to a list              
            # Keep count of total number of events and the relative number (ID) in the chunk
            eventID += 1
            eventsum += 1 
            

            #print("In While loop. eventsum = ", eventsum, "eventID = ", eventID, "eventChunk = ", eventChunk)
            if eventID == eventChunk:                
                yield eventMatrix
                eventID = 0 # reset this counter, after the yield, so on next() call it will run upto eventID again. 
                eventMatrix = [] # as I want the next chunk, re-initalise it here.  
                print("Got whole chunk. if eventID == eventChunk. eventsum = ", eventsum, "eventID = ", eventID, "eventChunk = ", eventChunk)


    f.close()
    print(" I am at EoF, have read in all data. File closed and generator exhausted. event = next(ReadEventGen) ")
    yield eventMatrix     # this yeild will be for the whole eventMatrix (if no chunk given) or for the (last) partial chunk, when EoF is hit)
 


# The Main Code is from here
printme(" *** THE CODE BEGINS *** ")

#-----------#
fileDir = Path("E:\PyScripts\Pico")


filename = "Slits_15_F0_16usPreTrig.traces"    # short file Forward
#filename = "Slits_4p6_F0.traces"              # biggest file I took

#fileDir = Path("C:/Users/opw44059/Desktop/Pico/EMU_Digitiser_March2020_Bk")
#filename = "CHA_50 ChB_95 ChC_92 ChD_89_Back_RingMid_Slits_15_Field0_16usPreTrig.traces"
#-----------#

#-----------#
#HiFi_Data_2020

#fileDir = Path("C:/Users/opw44059/Documents/Pico/HiFi_Data")
#filename = "HiFi_DigitialMuons_FirstData_short_run.traces"    # short file Forward
#filename = "HiFi_17Sept20_Slit10_ZF_Ch1to4.traces"    #
#filename = "HiFi_17Sept20_Slit100_ZF_C1to4.traces"    #
#filename = "HiFi_17Sept20_Slit40_ZF_C1to4.traces"    #
#filename = "HiFi_17Sept20_Slit20_TF20_C1to4.traces"    #







#ARGUS data
#fileDir = Path("C:/Users/opw44059/Desktop/Pico/ARGUS_Feb2020")
#filename = "401_402_403_trig_.traces"
#filename = "32_66_97_49.traces"

#Old EMU Data Upstairs
#fileDir = Path("C:/Users/opw44059/Desktop/Pico/TracesData")
#filename = "EMU_Ch34_H5505_p2050V_MuonON_Slits4p6_100.traces" # small file ch 34
#filename = "EMU_Ch34_H5505_p2050V_MuonON_Slits4p6_10000.traces" # big file ch 34
#filename = "EMU_Ch34_H5505_p2150V_MuonON_Slits4p6_10000.traces" # small file file ch 34 +100V HT

#filename = "EMU_Ch78_H5505_p2270V_MuonON_Slits4p6.traces" # small file ch 78
#filename = "EMU_Ch78_H5505_p2270V_MuonON_Slits4p6_100000.traces" # big file ch 78

#filename = "EMU_Ch31_H5505_p2200V_MuonON_Slits4p6.traces" # small file ch 31
#filename = "EMU_Ch31_H5505_p2200V_MuonON_Slits4p6_100000.traces" # big file ch 31

#fileDir = Path("C:/Users/opw44059/Documents/Pico/TracesData")
#filename = "EMU_Ch34_H5505_p2150V_MuonON_Slits4p6_100.traces"



#-----------#

fileDir = Path("C:/Users/opw44059/Documents/Pico/TracesData")

filepath = fileDir / filename   # the way to make paths using python Path function. Its a bit fussy. 

# Does a check of file and crashes code here if no file.
# Returns the EoF from f.tell
# TOdo - EoF isn't used as a criteria to continue. use is or remove it.
EoFLocation = FileExistChecker(filepath)


#Now define the ReadChunk Generator and a vairable for ChunkSize
# This is how many events (each including n channels) are requested to be read into memory. 
# If you are near EoF this will be less. Or if not passed will be whole file size. 
MyChunkSize = 500
trueChunkSize = 0

#Create and instance of the ReadChunk Generator
ChunkGen = ReadChunk(filepath,MyChunkSize)

# If you want to read the whole file, leave out the MyChunkSize argument. 
# I re-define MyChunkSize to be the length of the data array actually read, to avoid errors later. 
#ChunkGen = ReadChunk(filepath)

start = timer()    

# Read the header and assign important vairabiles to a list and global variables. 
# Get the next chunk of data (or i fno chunk is given, whole file)

# Now define the time (data_x  array)
# Erik's code: for (int j = 0; j < NbrSamples; j++) : sampleStr = (j * SampInterval).ToString("E4");
 
framesTotal = 0
ChunkID = 0
amps_times_all = [[],[],[],[],[],[],[],[]] # nested list for hits amplitudes - entire file

print("The main loop Starts.")

try:
    # read the file, taking eventMatrix worth from ChunkGen until all read
    for eventMatrix in ChunkGen:
    #for i in [0]:  
        
        ChunkID +=1
        data_x = np.arange(0,NbrSamples*SampInterval, SampInterval) * 1E6
            
        # convert to np array
        # I am hoping this is converted to np and re-assigned to the same variable name, meaning the memory 
        # for the python list version (which was quicker to load) is now freed. 
        eventMatrix = np.array(eventMatrix)
        
        #trueChunkSize needs to be the actual number (triggers / frames) read in, not the number requested
        #This is less when you are near EoF only.
        trueChunkSize = eventMatrix.shape[0]
        framesTotal += trueChunkSize # keep running total of frames
        
        print("Have now got the Chunk (eventMatrix)")
        print("MyChunkSize = ", MyChunkSize, " trueChunkSize = ", trueChunkSize, "framesTotal = ", framesTotal, "ChunkID  = ", ChunkID)

        
        # I want to completly analysis and plot the data in one channel, before moving onto the next.        
        # itterate over channels
 
        for chanID in range(NbrChanEnab): 
            plt.close("all")
            print("Main loop ChanID             = ", chanID)
       
            
            for event in range(trueChunkSize):
                
                data_y = -1 * eventMatrix[event,chanID,:] # PeakUtils works on positve polarity data only
                        
                # Using PeakUtils. This returns one list of indexes where the peak is. These are the sample number. 
                # To find the time or amplitude use this list of indexes into the time or voltage data.
                
                low_threshold = 0.01 # set this low to see all the low events in the PHS
                disc_threshold = 0.030 # set this as the instrument discriminator threshold 50mV for example
                
                indexes = peakutils.indexes(data_y, thres=low_threshold, min_dist=10, thres_abs=True)
                
                # make two lists, of times (in ns) and amplitudes (mV)
        
                hit_amp_temp = data_y[indexes]     # temp arrays for the hits, re-initalised each event loop  
                hit_time_temp = data_x[indexes]    # temp arrays for the hits, re-initalised each event loop   
                
                listLoc = chanID * 2
                amps_times_all[listLoc].extend(hit_amp_temp)
                amps_times_all[listLoc+1].extend(hit_time_temp)

                #Aim to make n plots, repeated every time a chunk is taken from file.                                    
                #Initalise my plot here.
                             
                nPlot = 10
                
                #One 'if' required to just make a few plots - not for every trace.
                if event <= nPlot and event <= trueChunkSize:        
                    
                    #Make arrays for the hits over discriminator
                    amplitudes_disc = [] # for the events over discriminator threshold - amplitudes
                    times_disc = []  # for the events over discriminator threshold - times
                    for i in range(hit_amp_temp.size): # itterate over the hits found            
                        if hit_amp_temp[i] >= disc_threshold:
                            amplitudes_disc.append(hit_amp_temp[i]) # the event amplidues for those above the software discriminator  
                            times_disc.append(hit_time_temp[i]) # the event times for those above the software discriminator
                   
                    title = "Analysis of file" + str(chanID) + filename[:-7] 
                    fig = plt.figure(title,figsize=(15,10))
                    fig.patch.set_facecolor('w')
                    OneTracePlt = fig.add_subplot(1,1,1)               
                    OneTracePlt.title.set_text('One Trace with Events Over Disciminator Threshold. Ch ' + str(chanID))  
                    OneTracePlt.plot(data_x, data_y, color='black', marker='.', linestyle='solid', linewidth=1, markersize=0.01)
                    #plt.scatter(x, y, s=80, facecolors='none', edgecolors='r')
                    OneTracePlt.plot(times_disc, amplitudes_disc, color='red', marker='*', linestyle = 'None')
                    OneTracePlt.plot(hit_time_temp, hit_amp_temp,color='green', marker='o', mfc='none', linestyle = 'None')
                    OneTracePlt.axhline(y=disc_threshold, xmin=data_x[0], xmax=data_x[-1],color='r', linestyle='--' )
                    OneTracePlt.set_ylabel('Volts [V] ')	
                    OneTracePlt.set_xlabel('Time [\u03bcs]')	                   
                    
                   #plt.tight_layout()
                    
                    
                    TempTitleString = filename[:-7] + "_Ch_" + str(chanID) + "Frm_" + str(framesTotal + event - trueChunkSize ) + "Disc_" + str(disc_threshold)
                    fig_name =  TempTitleString + ".png"   
                    
                    fig_path = fileDir / "figs" / fig_name
                    plt.savefig(fig_path)        
                    #print(" figures have been saved here = ", fig_path)
                    plt.show()
                    plt.close("all")
                 

    
    
    for chanID in range(NbrChanEnab): 
        print("ChanID             = ", chanID)
        plt.close("all")
        
        
        TempTitleString = filename[:-7] + "_Ch" + str(chanID) + "Frm" + str(framesTotal) + ".txt"  
        save_path = fileDir / "figs" / TempTitleString        
        myHeader = str(framesTotal) 
        dataout = np.array([amps_times_all[chanID*2], amps_times_all[(chanID*2)+1]]).T #transpose data, so to have it in two columns
        
        #Open the file with 'with' to endsure it closes properly.
        with open(save_path, 'w+') as datafile_id:              
            np.savetxt(datafile_id, dataout, header=myHeader, delimiter=',',newline= "\n")

        #I make a PHS and lieftime spectra here as all events are in memory.
        #PHS first, so use the 'amps' data from my list, thats position 0, 2, 4, 6

        my_bins = 1000        
        
        pico_PHS_temp, PHS_bin_edges = np.histogram(amps_times_all[chanID*2], density=False, bins = my_bins)    
        
        TempTitleString = filename[:-7] + "Ch_" + str(chanID) + "Frm_" + str(framesTotal) + "Disc_" + str(disc_threshold)
        save_name = TempTitleString + "_PHS.txt"   
        save_path = fileDir / "figs" / save_name   
        dataout = np.vstack((pico_PHS_temp[:],PHS_bin_edges[0:-1])).T
       
        with open(save_path, 'w+') as datafile_id:
        #Open the file with 'with' to endsure it closes properly.
            np.savetxt(datafile_id, dataout, delimiter=',',newline= "\n")  
                   
        #Lifetime use the 'time' data from my list, thats position 1, 3, 5, 7
        pico_Lifetime_temp, Lifetime_bin_edges = np.histogram(amps_times_all[(chanID*2)+1], density=False, bins = my_bins)
            
        dataout = np.vstack((pico_Lifetime_temp[:],Lifetime_bin_edges[0:-1])).T
        save_name = TempTitleString + "_Ltime.txt" 
        save_path = fileDir / "figs" / save_name  
              
        with open(save_path, 'w+') as datafile_id:
        #Open the file with 'with' to endsure it closes properly.
            np.savetxt(datafile_id, dataout, delimiter=',',newline= "\n") 
        print("Number of chunks processed  =", ChunkID)
        
        title = "Analysis of file" + str(chanID) + filename[:-7] 
        fig = plt.figure(title,figsize=(13, 10))
        fig.patch.set_facecolor('w')
        # tweak the spacings of the subplots, particarly to get the axis labels not to clash,
        fig.subplots_adjust(left=None, bottom=None, right=None, top=0.99, wspace=None, hspace=0.25)
               
        ToFPlot = plt.subplot(2,1,1)  
        ToFPlot.grid()
        ToFPlot.title.set_text('ToF Histogramme')             
        ToFPlot.bar(Lifetime_bin_edges[:-1], pico_Lifetime_temp, width=np.diff(Lifetime_bin_edges), edgecolor="black", align="edge")
        ToFPlot.set_ylabel('Counts [Count / time bin] ')	
        ToFPlot.set_xlabel('Time [us]')	
        
         
        PHSPlot = plt.subplot(2,1,2) 
        PHSPlot.grid()
        PHSPlot.title.set_text('Pulse Height Spectra')      
        PHSPlot.bar(PHS_bin_edges[:-1], pico_PHS_temp, width=np.diff(PHS_bin_edges), edgecolor="black", align="edge")
        PHSPlot.set_ylabel('Counts [Count / time bin]' )	
        PHSPlot.set_xlabel('Volts [V] ')	
        PHSPlot.axvline(x=disc_threshold, color='r', linestyle='-' )
        plt.tight_layout()
        fig_name =  TempTitleString + ".png"   
        
        fig_path = fileDir / "figs" / fig_name
        plt.savefig(fig_path)           
        plt.close("all")

except Exception as e:
    # here to catch the stop Itteration issued from exhausted generators
    print("When calling next() on generator ReadChunk caught: ", e)

  
    


theTime = (timer() - start)    

print("Run time                 = ", theTime)
print("Requested number of events (chunk size = )  = ", trueChunkSize)



printme("Code complete. Have a nice cup of tea.")   


