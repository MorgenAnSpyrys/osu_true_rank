poorly written tool to do some screening shenanigans

it fetches bonuspp from the api regardless of your choice at the start because it's already doing those api calls anyways to get the top100 scores and I was too lazy to properly separate it (even though api fetching bonus pp is not recommended)


to run add elitebotix /osu-host tournamenttopplays output csv path as arg 1, apiv1 key as arg 2


for elitebotix /osu-host tournamenttopplays, you MUST run it with amount:100 to get accurate results. I also HIGHLY recommend onlyranked:true, as some unranked maps that break pp are played in gimmick tournaments

i recommend defaulting to 416.6 bonus pp at runtime because derankers have artificially less bonus pp, but if you don't like that, that is an option as well

# step-by-step guide (for windows users):
#### to get your etx csv, you first need a text document containing osu user ids in each row
![image](https://user-images.githubusercontent.com/111236657/225871566-c6d62cd6-c0c8-480b-8faa-2e71c80b2a24.png)
#### next, you go to private messages with elitebotix (Elitebotix#2307 on discord)
#### type your osu-host command, this requires! amount:100 for accurate results and I highly recommend also using onlyranked:true
#### your command will look something like this:
![image](https://user-images.githubusercontent.com/111236657/225872028-94980cd8-9939-4d4e-ae73-3901e9150e16.png)
#### after processing, etx will send you a file called "tournament-topplays.csv"
![image](https://user-images.githubusercontent.com/111236657/225872334-f04c0b8e-89d8-4c71-99e1-36ae3207c693.png)
#### download and save this file
#### Download the executable from the releases tab. I've only built an exe because I'm lazy and Linux users are usually smart enough to build the binaries themselves
#### Open any terminal, I recommend PowerShell (via Windows Terminal if possible), by ctrl+right-clicking in the folder you saved the executable and clicking open in terminal/open command prompt here. Alternatively, open powershell directly from your start menu and manually navigate to your folder using cd.
![image](https://user-images.githubusercontent.com/111236657/225874453-062fd2ff-f703-4ead-a87a-1f7397dfa562.png)
#### right-click your csv in windows explorer and click "copy as path" or press ctrl+shift+c
![image](https://user-images.githubusercontent.com/111236657/225874858-cd0201ce-5595-4667-8d87-92aaa3903acb.png)
#### now head back to your terminal. type the local path of your executable (this should be .\osu_true_rank.exe, use tab to autocomplete.
![image](https://user-images.githubusercontent.com/111236657/225875263-8cbcdad0-5b90-4c95-aa96-9c62ce882945.png)
#### make sure there is a space behind the .exe, then right-click to paste your path, make sure it is surrounded by quotation marks
![image](https://user-images.githubusercontent.com/111236657/225875550-12c97b46-3da8-41e9-800c-4202fb850330.png)
#### next, head to https://osu.ppy.sh/p/api and copy your api key
#### as with the .exe, ensure there is a space after the last quotation mark, then right-click to paste the api key (no quotation marks this time!)
  ![image](https://user-images.githubusercontent.com/111236657/225876120-9e04a46a-a292-412b-8f69-70488ba77820.png)
#### press enter to run!
#### the program will ask you if you want to default to 416.6 bonus pp for all players, or fetch from the api
![image](https://user-images.githubusercontent.com/111236657/225876261-40546416-a596-423d-b3d4-19df53eadfa8.png)
#### I recommend defaulting to 416 as derankers have artificially lower bonus pp and this cannot be accounted for otherwise.
#### To default to 416, type Y or y and press enter, otherwise type N or n and press enter
#### the program will now run
![image](https://user-images.githubusercontent.com/111236657/225876588-a1dcd373-9295-4aeb-8b79-630e0faa3202.png)
#### upon completion of the file write, the program will tell you the path it has written to (it writes to the folder the executable is placed in)
![image](https://user-images.githubusercontent.com/111236657/225876869-9a4f0616-6b4d-413d-9982-2d658580ed35.png)

#### Here's the resulting output file:
![image](https://user-images.githubusercontent.com/111236657/225877074-d46df16b-fd28-4335-bfe6-8ba68d774c19.png)
#### column a contains the user id, column b the "corrected" weighted pp, and column c contains a semicolon (;) separated list of their 100 "true" top plays

[output.csv](https://github.com/MorgenAnSpyrys/osu_true_rank/files/11000229/output.csv)


