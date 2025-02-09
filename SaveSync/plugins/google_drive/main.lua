function Info()
	return {
		name = "Google Drive",
		description = "Google Drive binding",
		author = "Zachareee",
		icon_url = "https://upload.wikimedia.org/wikipedia/commons/1/12/Google_Drive_icon_%282020%29.svg",
	}
end

function Init(credentials)
	local gdrive = require("gdrive")
	print("This is real")
	print(gdrive.new({}))
end

--- Sync function
--- @param tag string Tagname for the folder
--- @param filename string Filename to upload
--- @param zipbuffer string Buffer of the zipfile created
function Upload(tag, filename, zipbuffer)
	local file = io.open("fake.zip", "wb")
	if file then
		file:write(zipbuffer)
		file:close()
	end
end

function Download() end
