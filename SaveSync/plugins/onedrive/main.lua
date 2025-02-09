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
	return "ONEDRIVECREDENTIALS"
end

function Abort()
	print("Onedrive has been aborted")
	return "Unsuccessful abort"
end
