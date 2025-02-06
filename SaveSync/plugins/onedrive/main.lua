function Info()
	return {
		name = "OneDrive",
		description = "OneDrive plugin",
		author = "Zachareee",
	}
end

function Init(credentials)
	print(credentials)

	-- simulate delay of authenticating
	os.execute('powershell "sleep 1"')
	return "ONEDRIVECREDENTIALS"
end

function Abort()
	print("Onedrive has been aborted")
	return "Unsuccessful abort"
end
