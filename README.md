# mode1

## main

- send command

# mode2

## main

- receive command
- send command to webkit thread

## web server

## webkit thread

# steps

1. run the daemon first by `www init`

   - it will create a web server
   - it will create a gtk application
   - it will listen the unix socket messages

2. run the command `www create --directory path \
--monitor HDMI-0 \
--anchors top,right \
--margin-top 5 \
--margin-right 5 \
--layer bottom \
--exclusive-zone \
--default-width 100 \
--default-height 100 \
--click-though true \
--keyboard-mode false \
--tags bar
`
   return the id of the created widget instance

3. run the command `www manage --id [id] reload` to reload the application
4. run the command `www manage --id [id] update --monitor` to update parameters
5. run the command `www manage --id [id] delete` to shutdown widget instance
